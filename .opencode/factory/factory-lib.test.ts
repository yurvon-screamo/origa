// factory-lib.test.ts — тесты чистых функций фабрики (bun test)
import { test, expect, describe } from "bun:test"
import {
  AGENT_MARK,
  isUserComment,
  newUserComments,
  untrackedIssues,
  kickoffPrompt,
  wakeupPrompt,
  worktreePaths,
  mergeState,
  type IssueComment,
  type IssueRef,
  type FactoryState,
} from "./factory-lib"

const c = (id: string, body: string, author = "yurvon-screamo"): IssueComment => ({
  id,
  author,
  body,
  createdAt: new Date(Date.UTC(2026, 9, 12, 10, Number(id))).toISOString(),
})

describe("isUserComment", () => {
  test("наши 🤖-комменты не триггерят", () => {
    expect(isUserComment(c("1", "🤖 Отчёт по #42"))).toBe(false)
    expect(isUserComment(c("2", "  🤖 вопрос"))).toBe(false)
  })
  test("служебные [factory] не триггерят", () => {
    expect(isUserComment(c("3", "[factory] wakeup"))).toBe(false)
  })
  test("юзерские триггерят", () => {
    expect(isUserComment(c("4", "поправь отступ"))).toBe(true)
  })
})

describe("newUserComments", () => {
  const comments = [
    c("10", "🤖 отчёт"),
    c("11", "сделай так"),
    c("12", "[factory] служебное"),
    c("13", "и ещё вот это"),
  ]
  test("только новые юзерские, по порядку создания", () => {
    const lastSeen = comments[0].createdAt
    const fresh = newUserComments(comments, lastSeen)
    expect(fresh.map((x) => x.id)).toEqual(["11", "13"])
  })
  test("пусто, если всё старое", () => {
    expect(newUserComments(comments, comments[3].createdAt)).toEqual([])
  })
})

describe("untrackedIssues", () => {
  const issues: IssueRef[] = [
    { number: 1, title: "a", body: "", state: "open", updatedAt: "" },
    { number: 2, title: "b", body: "", state: "open", updatedAt: "" },
  ]
  test("новые = не в состоянии", () => {
    const state: FactoryState = { issues: { "1": { sessionID: "s", worktree: "w", branch: "b", lastCommentAt: "", createdAt: 0 } } }
    expect(untrackedIssues(issues, state).map((i) => i.number)).toEqual([2])
  })
})

describe("kickoffPrompt", () => {
  const issue: IssueRef = { number: 42, title: "Сделай X", body: "Тело задачи", state: "open", updatedAt: "" }
  const p = kickoffPrompt(issue, "yurvon-screamo/origa")
  test("несёт ишью, пайп, шаблон отчёта и правила", () => {
    expect(p).toContain("#42")
    expect(p).toContain("Сделай X")
    expect(p).toContain("Тело задачи")
    expect(p).toContain("Closes #42")
    expect(p).toContain("code-quality-reviewer")
    expect(p).toContain("Что сделано")
    expect(p).toContain(AGENT_MARK)
    expect(p).toContain("Не мержи сам")
    expect(p).toContain("1280x800")
    expect(p).toContain("390x844")
  })
})

describe("wakeupPrompt", () => {
  test("квитирует комменты и напоминает канал ответа", () => {
    const p = wakeupPrompt(42, [c("11", "поправь"), c("13", "вот это")])
    expect(p).toContain("[factory]")
    expect(p).toContain("поправь")
    expect(p).toContain("вот это")
    expect(p).toContain("gh issue comment")
  })
})

describe("worktreePaths", () => {
  test("ветка и путь по номеру ишью", () => {
    expect(worktreePaths("/wt/root", 42)).toEqual({
      branch: "work/issue-42",
      worktree: "/wt/root/issue-42",
    })
  })
})

describe("mergeState", () => {
  test("последняя запись побеждает", () => {
    const a: FactoryState = { issues: { "1": { sessionID: "s1", worktree: "w", branch: "b", lastCommentAt: "", createdAt: 0 } } }
    const b: FactoryState = { issues: { "2": { sessionID: "s2", worktree: "w", branch: "b", lastCommentAt: "", createdAt: 0 } } }
    const m = mergeState(a, b)
    expect(Object.keys(m.issues).sort()).toEqual(["1", "2"])
  })
})
