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

/** Комменты агента фабрики идут от GitHub App uwuwu-agent[bot]. */
export const GH_AS_AGENT = "/home/yurvon/origa/.opencode/factory/gh-as-agent"

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
    "   (готовый markdown-файл + gh-as-agent issue comment с --body-file, первая строка обязательно " + AGENT_MARK + ")",
    `   и ЗАВЕРШИ ход. Ответ придёт новым комментом в ишью — тебя разбудят.`,
    "3. Если всё ясно: прогони план через code-quality-reviewer сабагента, исправь замечания.",
    "4. Реализуй. Коммить атомарно, сообщения по конвенции репо. Пуш ветки — как настроено (от бота).",
    "5. Если задача затрагивает UI/верстку: сними скрины ИНСТРУМЕНТОМ (свой скрипт писать нельзя —",
    "   он копипастит логин, онбординг и гоняет консоль загрузки):",
    "   cd end2end && npm ci          # в свежем ворктре нет node_modules",
    "   npm run shots -- list         # каталог экранов: выбери нужный --scenario <id>",
    "   npm run shots -- capture --scenario <id> --label issue-N --upload",
    "   Пререквизит: каталог корпуса ORIGA_CDN_CACHE_DIR — в ворктре нет cdn/ (в .gitignore),",
    "   нужен symlink на чекаут разработчика; без него инструмент возьмёт контент с прод-CDN.",
    "   Инструмент сам проверит окружение и напечатает markdown со ссылками на CDN.",
    "   Скрины НЕ коммитить в репо — бинарники в истории не оставляем никогда.",
    "   Если UI в задаче не затронут — в отчёте напиши «UI не затронут».",
    "6. Вставь напечатанные ссылки в отчёт и тело ПР (markdown: ![desktop](URL)).",
    `7. Отчёт в ишью: ${GH_AS_AGENT} issue comment ${issue.number} --body-file <файл> по ШАБЛОНУ ниже.`,
    `8. PR: ${GH_AS_AGENT} pr create --title "<кратко>" --body-file <файл> с «Closes #${issue.number}», отчётом и скринами (URL-ами).`,
    "   СРАЗУ после создания ПР переименуй свою сессию тулом session_rename:",
    '   tools.opencode.session_rename({ title: "PR #<номер ПР> · <название ПР>" })',
    `9. Проверь CI (${GH_AS_AGENT} pr checks --watch может висеть — опрашивай напрямую).`,
    "   Красный — чини. Зелёный CI ≠ готово: для ПР с кодом origa дождись",
    "   coverage-комментария qltysh[bot] (~10 мин после CI) и прими его:",
    "   непокрытые изменённые строки — тест или обоснование в отчёте.",
    "10. Мержит пользователь. Замечания придут комментами в ишью — исправляй в этой же ветке и пушь.",
    "",
    "ШАБЛОН ОТЧЁТА:",
    AGENT_MARK + " Отчёт по #" + issue.number,
    "**Что сделано:** …",
    "**Как проверить:** … (шаги)",
    "**Скрины:** ![desktop](https://s3.origa.uwuwu.net/…) ![mobile](…) (или «UI не затронут»)",
    "**Риски:** …",
    "",
    "ПРАВИЛА:",
    `- ВСЕ постинги в GitHub — только через ${GH_AS_AGENT} (выступаешь как uwuwu-agent[bot]).`,
    "  Голый gh для issue comment/pr create/pr checks --repo ЗАПРЕЩЁН (постить от аккаунта юзера нельзя).",
    "- КАЖДЫЙ твой комментарий в ишью начинается с " + AGENT_MARK + " — без него тебя разбудит собственный коммент.",
    "- ВСЁ, что должен увидеть пользователь — ТОЛЬКО комменты в ишью. Твой текст в сессии пользователь не читает.",
    "- Не мержи сам. Не закрывай ишью сам. Не выходи за пределы ворктри.",
    `- Разрешено: ${GH_AS_AGENT} issue comment/view, ${GH_AS_AGENT} pr create/checks. Запрещены: pr merge, issue close.`,
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
    "Отреагируй по пайпу: ответь на вопросы/исправь замечания. Ответ пользователю — gh-as-agent issue comment (🤖), не текст в сессии.",
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
