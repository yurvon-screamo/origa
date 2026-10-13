// factory-lib.ts — чистые функции origa-factory: состояние, комменты, промпты.
// Без зависимостей — импортируется демоном и тестами.

export interface IssueRef {
  number: number
  title: string
  body: string
  state: "open" | "closed"
  updatedAt: string
}

export interface IssueComment {
  /** GitHub-строка вида IC_… (НЕ число) */
  id: string
  /** login автора */
  author: string
  body: string
  /** ISO-время — надёжный порядок (IC-id лексически не сортируется) */
  createdAt: string
}

/** Состояние одной задачи фабрики. */
export interface TrackedIssue {
  sessionID: string
  worktree: string
  branch: string
  /** ISO-время последнего обработанного коммента */
  lastCommentAt: string
  /** id последнего обработанного коммента (created_at у GitHub секундный:
   *  сосед в ту же секунду отличаем по id) */
  lastCommentId: string
  createdAt: number
  /** неудачные попытки заведения (после MAX_ATTEMPTS — карантин до нового коммента) */
  attempts: number
}

export interface FactoryState {
  issues: Record<string, TrackedIssue>
}

/** Комменты агента фабрики — с этим префиксом, чтобы не петлить себя. */
export const AGENT_MARK = "🤖"

/** Авторы, чьи комменты мостятся агенту (репо публичное — allowlist обязателен). */
export function isAllowedAuthor(author: string, allowed: readonly string[]): boolean {
  return allowed.includes(author)
}

/** Комменты, которые триггерят агента: от allowlist-автора, не наши (🤖), не служебные. */
export function isUserComment(c: IssueComment, allowed: readonly string[]): boolean {
  if (!isAllowedAuthor(c.author, allowed)) return false
  const body = c.body.trimStart()
  if (body.startsWith(AGENT_MARK)) return false
  if (body.startsWith("[factory]")) return false
  return true
}

/**
 * Новые юзерские комменты после lastSeen, по порядку создания.
 * createdAt у GitHub секундный: порядок в одну секунду разрешаем позицией
 * id-границы в отсортированном списке (без потерь и без вечных дублей);
 * если пограничный коммент удалён — строгий fallback по времени.
 */
export function newUserComments(
  comments: IssueComment[],
  lastSeen: { at: string; id: string },
  allowed: readonly string[],
): IssueComment[] {
  const users = comments
    .filter((c) => isUserComment(c, allowed))
    .sort((a, b) =>
      a.createdAt === b.createdAt ? a.id.localeCompare(b.id) : a.createdAt.localeCompare(b.createdAt),
    )
  if (lastSeen.at === "") return users
  const idx = users.findIndex((c) => c.id === lastSeen.id)
  if (idx >= 0) return users.slice(idx + 1)
  return users.filter((c) => c.createdAt > lastSeen.at)
}

/** Ишью, которых ещё нет в состоянии. */
export function untrackedIssues(
  issues: IssueRef[],
  state: FactoryState,
): IssueRef[] {
  return issues.filter((i) => state.issues[String(i.number)] === undefined)
}

/** Нейтрализация подделываемых границ в теле ишью (промпт-инъекция):
 *  любая строка, начинающаяся с ===, глушится — границу не подделать. */
export function sanitizeBoundary(text: string): string {
  return text.replace(/^\s*===.*$/gm, "≡")
}

/** Промпт-контракт пайпа для новой задачи. */
export function kickoffPrompt(issue: IssueRef, repo: string): string {
  return [
    `Ты — девелопер фабрики. Репозиторий: ${repo}. Задача: ишью #${issue.number} «${issue.title.replace(/[\r\n]+/g, " ")}».`,
    "",
    "=== ТЕЛО ИШЬЮ ===",
    sanitizeBoundary(issue.body || "(пусто)"),
    "=== КОНЕЦ ТЕЛА ===",
    "",
    "ПАЙП (строго по порядку):",
    "1. Изучи задачу и репо (ветка ворктри уже создана — работай здесь).",
    "2. Если для старта нужны решения пользователя — задай вопросы ОДНИМ комментарием",
    "   (готовый markdown-файл + gh issue comment с --body-file, первая строка обязательно " + AGENT_MARK + ")",
    `   и ЗАВЕРШИ ход. Ответ придёт новым комментом в ишью — тебя разбудят.`,
    "3. Если всё ясно: прогони план через code-quality-reviewer сабагента, исправь замечания.",
    "4. Реализуй. Коммить атомарно, сообщения по конвенции репо.",
    "5. Если задача затрагивает UI/верстку: сними Playwright-ом скрины вьюпортов",
    "   desktop 1280x800 и mobile 390x844, сохрани в .pr-screenshots/ и закоммить.",
    "   Если UI в задаче не затронут — в отчёте напиши «UI не затронут».",
    `6. Отчёт в ишью: gh issue comment ${issue.number} --body-file <файл> по ШАБЛОНУ ниже.`,
    `7. PR: gh pr create --title "<кратко>" --body-file <файл> с «Closes #${issue.number}», отчётом и скринами.`,
    "   СРАЗУ после создания ПР переименуй свою сессию тулом session_rename:",
    '   tools.opencode.session_rename({ title: "PR #<номер ПР> · <название ПР>" })',
    "8. Проверь CI (gh pr checks --watch может висеть — опрашивай gh pr checks напрямую).",
    "   Красный — чини. Зелёный — коммент «готово к ревью».",
    "9. Мержит пользователь. Замечания придут комментами в ишью — исправляй в этой же ветке и пушь.",
    "",
    "ШАБЛОН ОТЧЁТА:",
    AGENT_MARK + " Отчёт по #" + issue.number,
    "**Что сделано:** …",
    "**Как проверить:** … (шаги)",
    "**Скрины:** desktop / mobile (или «UI не затронут»)",
    "**Риски:** …",
    "",
    "ПРАВИЛА:",
    "- КАЖДЫЙ твой комментарий в ишью начинается с " + AGENT_MARK + " — без него тебя разбудит собственный коммент.",
    "- ВСЁ, что должен увидеть пользователь — ТОЛЬКО gh issue comment. Твой текст в сессии пользователь не читает.",
    "- Не мержи сам. Не закрывай ишью сам. Не выходи за пределы ворктри.",
    "- Разрешённые gh-команды: gh issue comment/view, gh pr create/checks. Запрещены: gh pr merge, gh issue close.",
    "- Если задача правит сам код фабрики (.opencode/factory/) — в ПР напомни: после мержа нужен",
    "  systemctl --user restart origa-factory (работающий демон не подхватит новый код сам).",
  ].join("\n")
}

/** Промпт-будильник на новые комменты юзера. */
export function wakeupPrompt(
  issueNumber: number,
  comments: IssueComment[],
): string {
  const quoted = comments
    .map((c) => `@${c.author}: ${c.body.trim()}`)
    .join("\n---\n")
  return [
    `[factory] В ишью #${issueNumber} новые комментарии от пользователя:`,
    "",
    quoted,
    "",
    "Отреагируй по пайпу: ответь на вопросы/исправь замечания. Ответ пользователю — gh issue comment (🤖), не текст в сессии.",
  ].join("\n")
}

/** Ветка и путь ворктри для ишью. */
export function worktreePaths(root: string, issueNumber: number): { branch: string; worktree: string } {
  return {
    branch: `work/issue-${issueNumber}`,
    worktree: `${root}/issue-${issueNumber}`,
  }
}

/** Валидация записи state.json (битое состояние не должно попадать в git/REST пути).
 *  Три легальных формата: полный (ses_ + пути), чистый карантин (всё пусто —
 *  сбои старта) и гибрид (sessionID="" + живые пути — сессия умерла, работа
 *  в ворктри сохраняется до решения юзера; пишут quiet-dead и restartIssue). */
export function isValidTracked(v: unknown): v is TrackedIssue {
  if (v === null || typeof v !== "object") return false
  const t = v as Record<string, unknown>
  if (
    typeof t.lastCommentAt !== "string" ||
    typeof t.lastCommentId !== "string" ||
    typeof t.createdAt !== "number" ||
    !(t.attempts === undefined || typeof t.attempts === "number")
  ) return false
  const sesOk = t.sessionID === "" || (typeof t.sessionID === "string" && t.sessionID.startsWith("ses_"))
  if (!sesOk) return false
  if (t.worktree === "" && t.branch === "") return true // чистый карантин
  return (
    typeof t.worktree === "string" && t.worktree.startsWith("/") &&
    typeof t.branch === "string" && t.branch.startsWith("work/issue-")
  ) // полный или гибрид
}
