// origa-factory — issue-driven фабрика разработки.
// Демон: поллит ишью с label `work` в yurvon-screamo/origa, на каждую поднимает
// сессию девелопера в своём ворктри от свежего master, мостит комменты юзера
// (allowlist-авторы) в сессию, агент сам постит вопросы/отчёты/PR через gh.
// Закрытие ишью — сессия и ворктри умирают. Запуск: systemd --user.
import { appendFileSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync, renameSync, existsSync, rmSync } from "node:fs"
import { homedir } from "node:os"
import { dirname, join } from "node:path"
import {
  newUserComments,
  untrackedIssues,
  kickoffPrompt,
  wakeupPrompt,
  worktreePaths,
  isValidTracked,
  type FactoryState,
  type IssueComment,
  type IssueRef,
  type TrackedIssue,
} from "./factory-lib"

const REPO = "yurvon-screamo/origa"
const LABEL = "work"
/** allowlist авторов: комменты остальных аккаунтов мост игнорирует (репо публичное) */
const ALLOWED_AUTHORS: readonly string[] = ["yurvon-screamo"]
const ORIGA = join(homedir(), "origa")
const WORKTREES_ROOT = join(homedir(), ".local/share/origa-factory/worktrees")
const STATE_DIR = join(homedir(), ".local/state/origa-factory")
const STATE_FILE = join(STATE_DIR, "state.json")
const LOCK_DIR = join(homedir(), ".local/state/origa-factory/lock")
const LOG_PATH = join(homedir(), ".local/share/opencode/log/origa-factory.log")
const SERVICE_REG = join(homedir(), ".local/state/opencode/service.json")
const POLL_SEC = 60
const MODEL = { providerID: "zai-coding-plan", id: "glm-5.3-flash" } as const
const SPAWN_TIMEOUT_MS = 60_000
const REST_TIMEOUT_MS = 15_000
const MAX_ATTEMPTS = 3
const ISSUE_LIMIT = 300

const log = (m: string): void => {
  try { appendFileSync(LOG_PATH, `${new Date().toISOString()} ${m}\n`) } catch {}
}

/** Ротация: >5MB → .old (перезапись предыдущего .old). */
function rotateLog(): void {
  try {
    if (!existsSync(LOG_PATH)) return
    if (statSync(LOG_PATH).size > 5 * 1024 * 1024) {
      rmSync(`${LOG_PATH}.old`, { force: true })
      renameSync(LOG_PATH, `${LOG_PATH}.old`)
    }
  } catch {}
}

// ——— одиночный инстанс: lock-каталог + pid, самоосвобождение по SIGTERM ———

function acquireLock(): boolean {
  try {
    mkdirSync(LOCK_DIR, { recursive: false })
  } catch {
    // каталог есть: жив ли владелец?
    try {
      const pid = Number(readFileSync(join(LOCK_DIR, "pid"), "utf-8").trim())
      if (Number.isInteger(pid)) {
        process.kill(pid, 0) // бросает, если процесс мёртв
        return false // живой конкурент
      }
    } catch {
      // pid-файла нет или процесс мёртв — захватываем брошенный lock
    }
    try {
      rmSync(LOCK_DIR, { recursive: true, force: true })
      mkdirSync(LOCK_DIR, { recursive: false })
    } catch {
      return false
    }
  }
  writeFileSync(join(LOCK_DIR, "pid"), String(process.pid))
  const release = (): void => {
    try { rmSync(LOCK_DIR, { recursive: true, force: true }) } catch {}
    process.exit(0)
  }
  process.on("SIGTERM", release)
  process.on("SIGINT", release)
  return true
}

// ——— состояние (валидация + атомарная запись) ———

function loadState(): FactoryState {
  try {
    const raw: unknown = JSON.parse(readFileSync(STATE_FILE, "utf-8"))
    const issues = (raw as { issues?: unknown }).issues
    const out: FactoryState = { issues: {} }
    if (issues !== null && typeof issues === "object") {
      for (const [k, v] of Object.entries(issues as Record<string, unknown>)) {
        if (Number.isInteger(Number(k)) && isValidTracked(v)) {
          out.issues[k] = { ...v, attempts: v.attempts ?? 0 }
        } else {
          log(`state: запись #${k} битая — пропущена`)
        }
      }
    }
    return out
  } catch {
    return { issues: {} }
  }
}

function saveState(state: FactoryState): void {
  mkdirSync(STATE_DIR, { recursive: true })
  const tmp = `${STATE_FILE}.tmp`
  writeFileSync(tmp, JSON.stringify(state, null, 2))
  renameSync(tmp, STATE_FILE)
}

// ——— процессы (gh/git) с таймаутом ———

async function sh(cmd: string[]): Promise<string> {
  const proc = Bun.spawn({ cmd, cwd: ORIGA, stdout: "pipe", stderr: "pipe" })
  const killer = setTimeout(() => proc.kill(), SPAWN_TIMEOUT_MS)
  const [out, err] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ])
  const code = await proc.exited
  clearTimeout(killer)
  if (code !== 0) throw new Error(`${cmd[0]} (${code}): ${(err.trim() || out.trim()).slice(0, 200)}`)
  return out.trim()
}

/** Команда, которой разрешено падать (например, удаление уже отсутствующего). */
async function shOk(cmd: string[]): Promise<boolean> {
  try {
    await sh(cmd)
    return true
  } catch {
    return false
  }
}

async function ghJson<T>(args: string[], fields: string, guard: (v: unknown) => T | null): Promise<T | null> {
  try {
    const out = await sh(["gh", ...args, "--json", fields])
    const parsed: unknown = JSON.parse(out === "" ? "{}" : out)
    return guard(parsed)
  } catch (e) {
    log(`gh ${args.join(" ")}: ${e}`)
    return null
  }
}

const asIssues = (v: unknown): IssueRef[] | null =>
  Array.isArray(v)
    ? v.filter(
        (i): i is IssueRef =>
          i !== null && typeof i === "object" &&
          typeof (i as IssueRef).number === "number" &&
          typeof (i as IssueRef).title === "string",
      )
    : null

interface GhComment {
  id?: unknown
  author?: unknown
  body?: unknown
  createdAt?: unknown
}

const asComments = (v: unknown): IssueComment[] | null => {
  if (v === null || typeof v !== "object") return null
  const arr: unknown = (v as { comments?: unknown }).comments
  if (!Array.isArray(arr)) return null
  const out: IssueComment[] = []
  for (const raw of arr as GhComment[]) {
    if (raw === null || typeof raw !== "object") continue
    if (typeof raw.id !== "string" || typeof raw.body !== "string" || typeof raw.createdAt !== "string") continue
    const author =
      raw.author !== null && typeof raw.author === "object" && typeof (raw.author as { login?: unknown }).login === "string"
        ? (raw.author as { login: string }).login
        : "?"
    out.push({ id: raw.id, author, body: raw.body, createdAt: raw.createdAt })
  }
  return out
}

const issueState = (v: unknown): "open" | "closed" | null => {
  if (v === null || typeof v !== "object") return null
  const s = (v as { state?: unknown }).state
  if (typeof s !== "string") return null
  const norm = s.toLowerCase()
  return norm === "open" || norm === "closed" ? norm : null
}

async function listOpenIssues(): Promise<IssueRef[]> {
  return (await ghJson(
    ["issue", "list", "--repo", REPO, "--label", LABEL, "--state", "open", "--limit", String(ISSUE_LIMIT)],
    "number,title,body,state,updatedAt",
    asIssues,
  )) ?? []
}

interface IssueView { state: "open" | "closed"; comments: IssueComment[] }

async function viewIssue(n: number): Promise<IssueView | null> {
  const raw = await ghJson(
    ["issue", "view", String(n), "--repo", REPO],
    "state,comments",
    (v): unknown => v,
  )
  if (raw === null) return null
  const state = issueState(raw)
  const comments = asComments(raw)
  if (state === null || comments === null) return null
  return { state, comments }
}

// ——— opencode REST (service.json): статусы проверяются, ошибки не молчат ———

async function restCall(path: string, init?: { method?: string; body?: unknown }): Promise<unknown> {
  const reg = JSON.parse(readFileSync(SERVICE_REG, "utf-8")) as { url?: string; password?: string }
  const base = (reg.url ?? "").replace("0.0.0.0", "127.0.0.1")
  const auth = "Basic " + Buffer.from(`opencode:${reg.password ?? ""}`).toString("base64")
  let res: Response
  try {
    res = await fetch(`${base}/api${path}`, {
      method: init?.method ?? "GET",
      headers: { authorization: auth, "content-type": "application/json" },
      body: init?.body === undefined ? undefined : JSON.stringify(init.body),
      signal: AbortSignal.timeout(REST_TIMEOUT_MS),
    })
  } catch (e) {
    // транспортная ошибка/таймаут — статус неизвестен
    throw Object.assign(new Error(`transport: ${e}`), { status: 0 })
  }
  const text = await res.text()
  if (!res.ok) {
    throw Object.assign(new Error(`HTTP ${res.status}: ${text.slice(0, 120)}`), { status: res.status })
  }
  if (text.trim() === "") return {}
  const j: unknown = JSON.parse(text)
  return (j as { data?: unknown }).data ?? j
}

async function createSession(worktree: string, issueNumber: number): Promise<string | null> {
  const raw = await restCall("/session", {
    method: "POST",
    body: {
      title: `factory #${issueNumber}`,
      model: { providerID: MODEL.providerID, id: MODEL.id },
      location: { directory: worktree },
    },
  })
  const id = raw !== null && typeof raw === "object" ? (raw as { id?: unknown }).id : undefined
  return typeof id === "string" && id.startsWith("ses_") ? id : null
}

async function promptSession(sessionID: string, text: string): Promise<boolean> {
  try {
    await restCall(`/session/${sessionID}/prompt`, { method: "POST", body: { text } })
    return true
  } catch (e) {
    log(`prompt ${sessionID}: ${e}`)
    return false
  }
}

type Liveness = "alive" | "dead" | "unknown"

/** 404 = сессии нет; таймаут/сеть/5xx = неизвестно — разрушать нельзя. */
async function sessionLiveness(sessionID: string): Promise<Liveness> {
  if (sessionID === "") return "dead"
  try {
    await restCall(`/session/${sessionID}`)
    return "alive"
  } catch (e) {
    const status = (e as { status?: number }).status
    if (status === 404 || status === 400) return "dead"
    return "unknown"
  }
}

async function deleteSession(sessionID: string): Promise<boolean> {
  try {
    await restCall(`/session/${sessionID}`, { method: "DELETE" })
    return true
  } catch (e) {
    log(`delete session ${sessionID}: ${e}`)
    return false
  }
}

// ——— жизненный цикл задачи ———

/** Сверка на старте: осиротевшие ворктри/ветки без state — вычистить. */
function reconcileOrphans(state: FactoryState): void {
  const known = new Set(Object.values(state.issues).map((t) => t.worktree))
  if (!existsSync(WORKTREES_ROOT)) return
  for (const name of readdirSafe(WORKTREES_ROOT)) {
    const full = join(WORKTREES_ROOT, name)
    if (!known.has(full) && /^issue-\d+$/.test(name)) {
      try {
        log(`reconcile: осиротевший ворктри ${full} — удаляю`)
        rmSync(full, { recursive: true, force: true })
        void shOk(["git", "-C", ORIGA, "worktree", "prune"])
        const m = name.match(/^issue-(\d+)$/)
        if (m) void shOk(["git", "-C", ORIGA, "branch", "-D", `work/${name}`])
      } catch (e) {
        log(`reconcile: ${full} не удался — ${e} (не критично, попробую на след. старте)`)
      }
    }
  }
}

function readdirSafe(p: string): string[] {
  try {
    return readdirSync(p)
  } catch {
    return []
  }
}

async function startIssue(issue: IssueRef, state: FactoryState, attemptsSeed: number): Promise<void> {
  const { branch, worktree } = worktreePaths(WORKTREES_ROOT, issue.number)
  mkdirSync(WORKTREES_ROOT, { recursive: true })

  // идемпотентность: хвосты прошлого неудачного старта вычищаем безусловно
  if (existsSync(worktree)) {
    log(`issue #${issue.number}: ворктри уже есть — чищу перед пересозданием`)
    rmSync(worktree, { recursive: true, force: true })
  }
  await shOk(["git", "-C", ORIGA, "worktree", "prune"])
  await shOk(["git", "-C", ORIGA, "branch", "-D", branch])

  await sh(["git", "-C", ORIGA, "fetch", "origin", "master"])
  await sh(["git", "-C", ORIGA, "worktree", "add", "-b", branch, worktree, "origin/master"])

  const sessionID = await createSession(worktree, issue.number)
  if (sessionID === null) throw new Error("сессия не создалась")

  const sent = await promptSession(sessionID, kickoffPrompt(issue, REPO))
  if (!sent) {
    await deleteSession(sessionID) // rollback: сессия создана, но не запущена
    throw new Error("kickoff-промпт не ушёл")
  }

  // актуальная граница комментов: не будить агента старыми
  const viewed = await viewIssue(issue.number)
  let lastCommentAt = ""
  let lastCommentId = ""
  if (viewed && viewed.comments.length > 0) {
    const last = viewed.comments.reduce((m, c) => (c.createdAt >= m.createdAt ? c : m), viewed.comments[0])
    lastCommentAt = last.createdAt
    lastCommentId = last.id
  }

  state.issues[String(issue.number)] = {
    sessionID,
    worktree,
    branch,
    lastCommentAt,
    lastCommentId,
    createdAt: Date.now(),
    attempts: attemptsSeed,
  }
  saveState(state)
  log(`issue #${issue.number}: сессия ${sessionID}, пайп запущен (попыток до этого: ${attemptsSeed})`)
}

/** Полная чистка; помечает задачу удалённой ТОЛЬКО при подтверждении каждого шага. */
async function finishIssue(n: number, tracked: TrackedIssue, state: FactoryState): Promise<void> {
  log(`issue #${n}: закрыта — чистка`)
  let clean = true
  const liveness = await sessionLiveness(tracked.sessionID)
  if (liveness === "unknown") {
    clean = false
    log(`issue #${n}: живость сессии неизвестна — ретрай чистки на след. тике`)
  } else if (liveness === "alive" && !(await deleteSession(tracked.sessionID))) {
    clean = false
    log(`issue #${n}: сессия не удалилась — ретрай чистки на след. тике`)
  }
  if (existsSync(tracked.worktree)) {
    const removed = await shOk(["git", "-C", ORIGA, "worktree", "remove", "--force", tracked.worktree])
    if (!removed) {
      rmSync(tracked.worktree, { recursive: true, force: true })
      await shOk(["git", "-C", ORIGA, "worktree", "prune"])
    }
    if (existsSync(tracked.worktree)) {
      clean = false
      log(`issue #${n}: ворктри не удалился — ретрай чистки на след. тике`)
    }
  }
  await shOk(["git", "-C", ORIGA, "branch", "-D", tracked.branch])
  if (clean) {
    delete state.issues[String(n)]
    saveState(state)
    log(`issue #${n}: чистка завершена`)
  }
}

// ——— цикл ———

/** Уведомление в ишью с одним ретраем (F2: потеря коммента недопустима). */
async function notifyIssue(n: number, body: string): Promise<void> {
  const args = ["gh", "issue", "comment", String(n), "--repo", REPO, "--body", body]
  if (await shOk(args)) return
  await new Promise((r) => setTimeout(r, 15_000))
  if (!(await shOk(args))) log(`issue #${n}: уведомление не доставлено — ${body.slice(0, 80)}`)
}

/** Учесть неудачную попытку (seed — счётчик ДО попытки, из записи рестарт-пути). */
async function registerFailure(n: number, state: FactoryState, seed: number, err: unknown): Promise<void> {
  const key = String(n)
  const attempts = seed + 1
  const entry: TrackedIssue = state.issues[key] ?? {
    sessionID: "", worktree: "", branch: "",
    lastCommentAt: "", lastCommentId: "", createdAt: Date.now(), attempts: 0,
  }
  entry.attempts = attempts
  state.issues[key] = entry
  saveState(state)
  log(`issue #${n}: старт не удался (попытка ${attempts}) — ${err}`)
  if (attempts >= MAX_ATTEMPTS) {
    await notifyIssue(n,
      `🤖 [factory] не смог завестись (попытка ${attempts}). Проверь лог ~/.local/share/opencode/log/origa-factory.log и напиши комментарий — попробую снова.`)
  }
}

async function tick(state: FactoryState): Promise<void> {
  const issues = await listOpenIssues()

  for (const issue of untrackedIssues(issues, state)) {
    try {
      await startIssue(issue, state, 0)
    } catch (e) {
      await registerFailure(issue.number, state, 0, e)
    }
  }

  for (const key of Object.keys(state.issues)) {
    const tracked = state.issues[key]
    const n = Number(key)
    const viewed = await viewIssue(n)
    if (viewed === null) continue
    if (viewed.state === "closed") {
      try {
        await finishIssue(n, tracked, state)
      } catch (e) {
        log(`issue #${n}: cleanup не удался — ${e}`)
      }
      continue
    }
    const fresh = newUserComments(viewed.comments, { at: tracked.lastCommentAt, id: tracked.lastCommentId }, ALLOWED_AUTHORS)

    if (tracked.sessionID === "") {
      // карантин: ждём явной активности юзера
      if (fresh.length === 0) continue
      await restartIssue(n, tracked, fresh, issues, viewed, state)
      continue
    }

    if (fresh.length === 0) {
      // юзер молчит, но сессия могла умереть посреди прогона — заметить (R3):
      // dead без fresh → уведомление и карантин (рестарт по комменту юзера,
      // несохранённая работа ворктри не стирается молча)
      const quiet = await sessionLiveness(tracked.sessionID)
      if (quiet === "dead") {
        log(`issue #${n}: сессия умерла (юзер молчит) — карантин + уведомление`)
        tracked.sessionID = ""
        saveState(state)
        await notifyIssue(n,
          "🤖 [factory] сессия разработки потеряна. Напиши комментарий — перезапущу задачу (несохранённые изменения ворктри будут перезаписаны с master).")
      }
      continue
    }

    const liveness = await sessionLiveness(tracked.sessionID)
    if (liveness === "unknown") {
      log(`issue #${n}: живость сессии неизвестна (сеть/таймаут) — пропускаю тик`)
      continue // разрушать живую сессию по неоднозначному сигналу нельзя
    }
    if (liveness === "dead") {
      log(`issue #${n}: сессия умерла — перезапуск задачи`)
      await restartIssue(n, tracked, fresh, issues, viewed, state)
      continue
    }
    log(`issue #${n}: ${fresh.length} новых коммент(ов) → сессия`)
    if (await promptSession(tracked.sessionID, wakeupPrompt(n, fresh))) {
      const last = fresh[fresh.length - 1]
      tracked.lastCommentAt = last.createdAt
      tracked.lastCommentId = last.id
      saveState(state)
    } else {
      log(`issue #${n}: промпт не ушёл — граница комментов не сдвинута`)
    }
  }
}

/** Перезапуск задачи: граница комментов двигается ДО старта (иначе петля
 *  на старых комментах), счётчик попыток передаётся явно и переживает путь. */
async function restartIssue(
  n: number,
  tracked: TrackedIssue,
  fresh: IssueComment[],
  issues: IssueRef[],
  viewed: { state: "open" | "closed"; comments: IssueComment[] },
  state: FactoryState,
): Promise<void> {
  const seed = tracked.attempts
  const last = fresh[fresh.length - 1]
  tracked.lastCommentAt = last.createdAt
  tracked.lastCommentId = last.id
  tracked.sessionID = "" // карантин до успеха старта
  saveState(state)
  const ref = issues.find((i) => i.number === n) ?? viewedToRef(n, viewed)
  try {
    await startIssue(ref, state, seed)
  } catch (e) {
    await registerFailure(n, state, seed, e)
  }
}

function viewedToRef(n: number, viewed: IssueView): IssueRef {
  return { number: n, title: `issue #${n}`, body: "(перезапуск после сбоя — см. комменты ишью)", state: viewed.state, updatedAt: "" }
}

let tickCount = 0

async function main(): Promise<void> {
  if (!acquireLock()) {
    console.error("другой живой инстанс origa-factory держит lock — выходим")
    process.exit(1)
  }
  mkdirSync(dirname(LOG_PATH), { recursive: true })
  rotateLog()
  log(`origa-factory: старт (repo=${REPO}, label=${LABEL}, poll=${POLL_SEC}s, model=${MODEL.id})`)
  const state = loadState()
  log(`восстановлено задач: ${Object.keys(state.issues).length}`)
  reconcileOrphans(state)
  while (true) {
    try {
      await tick(state)
    } catch (e) {
      log(`tick: ${e}`)
    }
    tickCount++
    if (tickCount % 10 === 0) rotateLog() // ~раз в 10 минут живого демона
    await new Promise((r) => setTimeout(r, POLL_SEC * 1000))
  }
}

void main()
