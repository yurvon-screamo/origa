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
  createdAt: number
}

export interface FactoryState {
  issues: Record<string, TrackedIssue>
}

/** Комменты агента фабрики — с этим префиксом, чтобы не петлить себя. */
export const AGENT_MARK = "🤖"

/** Комменты, которые триггерят агента: не наши (без 🤖) и не служебные [factory]. */
export function isUserComment(c: IssueComment): boolean {
  const body = c.body.trimStart()
  if (body.startsWith(AGENT_MARK)) return false
  if (body.startsWith("[factory]")) return false
  return true
}

/** Новые юзерские комменты после lastSeen (ISO-время), по порядку создания. */
export function newUserComments(
  comments: IssueComment[],
  lastSeen: string,
): IssueComment[] {
  return comments
    .filter((c) => c.createdAt > lastSeen && isUserComment(c))
    .sort((a, b) => a.createdAt.localeCompare(b.createdAt))
}

/** Ишью, которых ещё нет в состоянии. */
export function untrackedIssues(
  issues: IssueRef[],
  state: FactoryState,
): IssueRef[] {
  return issues.filter((i) => state.issues[String(i.number)] === undefined)
}

/** Промпт-контракт пайпа для новой задачи. */
export function kickoffPrompt(issue: IssueRef, repo: string): string {
  return [
    `Ты — девелопер фабрики. Репозиторий: ${repo}. Задача: ишью #${issue.number} «${issue.title}».`,
    "",
    "=== ТЕЛО ИШЬЮ ===",
    issue.body || "(пусто)",
    "=== КОНЕЦ ТЕЛА ===",
    "",
    "ПАЙП (строго по порядку):",
    "1. Изучи задачу и репо (ветка ворктри уже создана — работай здесь).",
    "2. Если для старта нужны решения пользователя — задай вопросы ОДНИМ комментарием:",
    `   gh issue comment ${issue.number} --body "${AGENT_MARK} <вопросы списком>"`,
    "   и ЗАВЕРШИ ход. Ответ придёт новым комментом в ишью — тебя разбудят.",
    "3. Если всё ясно: прогони план через code-quality-reviewer сабагента, исправь замечания.",
    "4. Реализуй. Коммить атомарно, сообщения по конвенции репо.",
    "5. Если задача затрагивает UI/верстку: сними Playwright-ом скрины вьюпортов",
    "   desktop 1280x800 и mobile 390x844, сохрани в .pr-screenshots/ и закоммить.",
    "   Если UI в задаче не затронут — в отчёте напиши «UI не затронут».",
    `6. Отчёт в ишью: gh issue comment ${issue.number} --body-file <файл> по ШАБЛОНУ ниже.`,
    `7. PR: gh pr create --title "<кратко>" --body-file <файл> с «Closes #${issue.number}», отчётом и скринами.`,
    "8. Проверь CI (gh pr checks --watch). Красный — чини. Зелёный — коммент «готово к ревью».",
    "9. Мержит пользователь. Замечания придут комментами в ишью — исправляй в этой же ветке и пушь.",
    "",
    "ШАБЛОН ОТЧЁТА:",
    `${AGENT_MARK} Отчёт по #${issue.number}`,
    "**Что сделано:** …",
    "**Как проверить:** … (шаги)",
    "**Скрины:** desktop / mobile (или «UI не затронут»)",
    "**Риски:** …",
    "",
    "ПРАВИЛА:",
    "- ВСЁ, что должен увидеть пользователь — ТОЛЬКО gh issue comment (префикс 🤖). Твой текст в сессии пользователь не читает.",
    "- Не мержи сам. Не закрывай ишью сам. Не выходи за пределы ворктри.",
    `- Доступныеgh-команды: gh issue comment/view, gh pr create/checks. Никаких gh pr merge, gh issue close.`,
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

/** Безопасный merge состояния (последняя запись побеждает, потери нет — файл мал). */
export function mergeState(prev: FactoryState, next: FactoryState): FactoryState {
  return { issues: { ...prev.issues, ...next.issues } }
}
