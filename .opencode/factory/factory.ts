// origa-factory — issue-driven фабрика разработки.
// Демон: поллит ишью с label `work` в yurvon-screamo/origa, на каждую поднимает
// сессию девелопера в своём ворктри от свежего master, мостит комменты юзера
// в сессию, агент сам постит вопросы/отчёты/PR через gh. Закрытие ишью —
// сессия и ворктри умирают. Запуск: systemd --user (origa-factory.service).
import { appendFileSync, mkdirSync, readFileSync, writeFileSync, existsSync } from "node:fs"
import { homedir } from "node:os"
import { join } from "node:path"
import {
  AGENT_MARK,
  newUserComments,
  untrackedIssues,
  kickoffPrompt,
  wakeupPrompt,
  worktreePaths,
  type FactoryState,
  type IssueComment,
  type IssueRef,
  type TrackedIssue,
} from "./factory-lib"

const REPO = "yurvon-screamo/origa"
const LABEL = "work"
const ORIGA = join(homedir(), "origa")
const WORKTREES_ROOT = join(homedir(), ".local/share/origa-factory/worktrees")
const STATE_DIR = join(homedir(), ".local/state/origa-factory")
const STATE_FILE = join(STATE_DIR, "state.json")
const LOG_PATH = join(homedir(), ".local/share/opencode/log/origa-factory.log")
const SERVICE_REG = join(homedir(), ".local/state/opencode/service.json")
const POLL_SEC = 60
const MODEL = { providerID: "zai-coding-plan", id: "glm-5.3-flash" } as const

const log = (m: string): void => {
  try { appendFileSync(LOG_PATH, `${new Date().toISOString()} ${m}\n`) } catch {}
}

// ——— состояние ———

function loadState(): FactoryState {
  try {
    const raw: unknown = JSON.parse(readFileSync(STATE_FILE, "utf-8"))
    const issues = (raw as { issues?: unknown }).issues
    if (issues !== null && typeof issues === "object") return { issues: issues as FactoryState["issues"] }
  } catch { /* нет/битый — новый */ }
  return { issues: {} }
}

function saveState(state: FactoryState): void {
  mkdirSync(STATE_DIR, { recursive: true })
  writeFileSync(STATE_FILE, JSON.stringify(state, null, 2))
}

// ——— gh (авторизация юзера, отдельный GH_TOKEN поддерживается env) ———

async function sh(cmd: string[]): Promise<string> {
  const proc = Bun.spawn({ cmd, cwd: ORIGA, stdout: "pipe", stderr: "pipe" })
  const [out, err] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ])
  const code = await proc.exited
  if (code !== 0 && err.trim() !== "") {
    throw new Error(`${cmd[0]}: ${err.trim().slice(0, 200)}`)
  }
  return out.trim()
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
    ["issue", "list", "--repo", REPO, "--label", LABEL, "--state", "open", "--limit", "100"],
    "number,title,body,state,updatedAt",
    asIssues,
  )) ?? []
}

async function viewIssue(n: number): Promise<{ state: "open" | "closed"; comments: IssueComment[] } | null> {
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

// ——— opencode REST (service.json) ———

interface RestSession { id: string }

async function restCall(path: string, init?: { method?: string; body?: unknown }): Promise<unknown | null> {
  try {
    const reg = JSON.parse(readFileSync(SERVICE_REG, "utf-8")) as { url?: string; password?: string }
    const base = (reg.url ?? "").replace("0.0.0.0", "127.0.0.1")
    const auth = "Basic " + Buffer.from(`opencode:${reg.password ?? ""}`).toString("base64")
    const res = await fetch(`${base}/api${path}`, {
      method: init?.method ?? "GET",
      headers: { authorization: auth, "content-type": "application/json" },
      body: init?.body === undefined ? undefined : JSON.stringify(init.body),
    })
    const text = await res.text()
    if (text.trim() === "") return {} // 204/пустой ответ (DELETE) — не ошибка
    const j: unknown = JSON.parse(text)
    return (j as { data?: unknown }).data ?? j
  } catch (e) {
    log(`rest ${path}: ${e}`)
    return null
  }
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
  const ok = await restCall(`/session/${sessionID}/prompt`, { method: "POST", body: { text } })
  return ok !== null
}

async function deleteSession(sessionID: string): Promise<void> {
  await restCall(`/session/${sessionID}`, { method: "DELETE" })
}

// ——— жизненный цикл задачи ———

async function startIssue(issue: IssueRef, state: FactoryState): Promise<void> {
  const { branch, worktree } = worktreePaths(WORKTREES_ROOT, issue.number)
  mkdirSync(WORKTREES_ROOT, { recursive: true })
  log(`issue #${issue.number}: ворктри ${worktree}`)
  await sh(["git", "-C", ORIGA, "fetch", "origin", "master"])
  await sh(["git", "-C", ORIGA, "worktree", "add", "-b", branch, worktree, "origin/master"])

  const sessionID = await createSession(worktree, issue.number)
  if (sessionID === null) {
    log(`issue #${issue.number}: сессия не создалась — откат ворктри`)
    await sh(["git", "-C", ORIGA, "worktree", "remove", "--force", worktree])
    return
  }

  const sent = await promptSession(sessionID, kickoffPrompt(issue, REPO))
  if (!sent) {
    log(`issue #${issue.number}: промпт не ушёл — очистка`)
    await deleteSession(sessionID)
    await sh(["git", "-C", ORIGA, "worktree", "remove", "--force", worktree])
    return
  }

  // актуальный lastCommentAt: не будить агента старыми комментами
  const viewed = await viewIssue(issue.number)
  const lastCommentAt = viewed && viewed.comments.length > 0
    ? viewed.comments.reduce((m, c) => (c.createdAt > m ? c.createdAt : m), "")
    : ""

  const tracked: TrackedIssue = {
    sessionID,
    worktree,
    branch,
    lastCommentAt,
    createdAt: Date.now(),
  }
  state.issues[String(issue.number)] = tracked
  saveState(state)
  log(`issue #${issue.number}: сессия ${sessionID}, пайп запущен`)
}

async function finishIssue(n: number, tracked: TrackedIssue, state: FactoryState): Promise<void> {
  log(`issue #${n}: закрыта — чистка`)
  await deleteSession(tracked.sessionID)
  if (existsSync(tracked.worktree)) {
    await sh(["git", "-C", ORIGA, "worktree", "remove", "--force", tracked.worktree]).catch(() => {})
  }
  await sh(["git", "-C", ORIGA, "branch", "-D", tracked.branch]).catch(() => {})
  delete state.issues[String(n)]
  saveState(state)
  log(`issue #${n}: чистка завершена`)
}

// ——— цикл ———

async function tick(state: FactoryState): Promise<void> {
  const issues = await listOpenIssues()

  for (const issue of untrackedIssues(issues, state)) {
    await startIssue(issue, state)
  }

  for (const key of Object.keys(state.issues)) {
    const n = Number(key)
    const tracked = state.issues[key]
    const viewed = await viewIssue(n)
    if (viewed === null) continue
    if (viewed.state === "closed") {
      await finishIssue(n, tracked, state)
      continue
    }
    const fresh = newUserComments(viewed.comments, tracked.lastCommentAt)
    if (fresh.length === 0) continue
    log(`issue #${n}: ${fresh.length} новых коммент(ов) → сессия`)
    const ok = await promptSession(tracked.sessionID, wakeupPrompt(n, fresh))
    if (ok) {
      tracked.lastCommentAt = fresh.reduce((m, c) => (c.createdAt > m ? c.createdAt : m), tracked.lastCommentAt)
      saveState(state)
    }
  }
}

async function main(): Promise<void> {
  log(`origa-factory: старт (repo=${REPO}, label=${LABEL}, poll=${POLL_SEC}s, model=${MODEL.id})`)
  const state = loadState()
  log(`восстановлено задач: ${Object.keys(state.issues).length}`)
  while (true) {
    try {
      await tick(state)
    } catch (e) {
      log(`tick: ${e}`)
    }
    await new Promise((r) => setTimeout(r, POLL_SEC * 1000))
  }
}

void main()
