import { createHash } from "node:crypto";

import type { ExamRecord } from "./examine.ts";

/**
 * The report over a run's exam traces and the GitHub issue drafts for
 * its findings.
 *
 * A finding is a flagged verdict. The issue body is rendered strictly
 * from verified fields (ids, numbers, file hashes) — the decision model
 * returns probabilities, not prose, and nothing user-visible is copied
 * into an issue without passing through a checked field.
 */
export interface Finding {
    readonly fingerprint: string;
    readonly scenario: string;
    readonly screen: string;
    readonly viewport: string;
    readonly locale: string;
    readonly questionId: string;
    readonly questionLabel: string;
    /** noul: p(true) of the defect. choice: probability of the WINNING option. */
    readonly pTrue: number;
    /** choice only: probability of the expected option(s). */
    readonly pExpected: number | null;
    readonly threshold: number;
    readonly imageFile: string;
    readonly imageSha256: string;
    readonly capturedAt: string;
    readonly contextOverflow: boolean;
}

/** Stable key: same defect on the same screen is the same finding, run after run.
 *
 * A trailing card counter (the dynamic review-body captures) is dropped:
 * the scheduler deals a different kind order every run, the defect class
 * belongs to the screen kind, not to the deal index.
 */
export function findingFingerprint(input: {
    scenario: string;
    screen: string;
    viewport: string;
    locale: string;
    questionId: string;
}): string {
    const screen = input.screen.replace(/-\d+$/, "");
    return createHash("sha256")
        .update([input.scenario, screen, input.viewport, input.locale, input.questionId].join("|"))
        .digest("hex")
        .slice(0, 16);
}

/** Every flagged verdict of the run, oldest capture first. */
export function collectFindings(records: readonly ExamRecord[]): Finding[] {
    const findings: Finding[] = [];
    for (const record of records) {
        for (const verdict of record.verdicts) {
            if (!verdict.flagged) continue;
            if (verdict.kind === "choice") {
                findings.push({
                    fingerprint: findingFingerprint({
                        scenario: record.screen.scenario,
                        screen: record.screen.screen,
                        viewport: record.screen.viewport,
                        locale: record.screen.locale,
                        questionId: `${verdict.questionId}=${verdict.expected.join("|")}`,
                    }),
                    scenario: record.screen.scenario,
                    screen: record.screen.screen,
                    viewport: record.screen.viewport,
                    locale: record.screen.locale,
                    questionId: verdict.questionId,
                    questionLabel: `${verdict.label}: ждали ${verdict.expected.join(" | ")}, увидели «${verdict.actual}» (p выигрыша=${(verdict.pWinner ?? 0).toFixed(3)}, p ожидаемой=${(verdict.pExpected ?? 0).toFixed(3)})`,
                    pTrue: verdict.pWinner ?? 0,
                    pExpected: verdict.pExpected,
                    threshold: 0,
                    imageFile: record.screen.file,
                    imageSha256: record.screen.sha256,
                    capturedAt: record.capturedAt,
                    contextOverflow: record.contextOverflow,
                });
                continue;
            }
            findings.push({
                fingerprint: findingFingerprint({
                    scenario: record.screen.scenario,
                    screen: record.screen.screen,
                    viewport: record.screen.viewport,
                    locale: record.screen.locale,
                    questionId: verdict.questionId,
                }),
                scenario: record.screen.scenario,
                screen: record.screen.screen,
                viewport: record.screen.viewport,
                locale: record.screen.locale,
                questionId: verdict.questionId,
                questionLabel: verdict.label,
                pTrue: verdict.pTrue ?? 0,
                pExpected: null,
                threshold: verdict.threshold,
                imageFile: record.screen.file,
                imageSha256: record.screen.sha256,
                capturedAt: record.capturedAt,
                contextOverflow: record.contextOverflow,
            });
        }
    }
    return findings;
}

export interface UnansweredQuestion {
    readonly scenario: string;
    readonly screen: string;
    readonly viewport: string;
    readonly questionId: string;
    readonly questionLabel: string;
}

/**
 * Questions the model left unanswered.
 *
 * A skipped question would read as «no defect found» — the report and
 * the exit code must not let that pass silently.
 */
export function collectUnanswered(records: readonly ExamRecord[]): UnansweredQuestion[] {
    const unanswered: UnansweredQuestion[] = [];
    for (const record of records) {
        for (const verdict of record.verdicts) {
            if (verdict.kind === "noul" && verdict.pTrue === null) {
                unanswered.push({
                    scenario: record.screen.scenario,
                    screen: record.screen.screen,
                    viewport: record.screen.viewport,
                    questionId: verdict.questionId,
                    questionLabel: verdict.label,
                });
            }
            if (verdict.kind === "choice" && verdict.actual === null) {
                unanswered.push({
                    scenario: record.screen.scenario,
                    screen: record.screen.screen,
                    viewport: record.screen.viewport,
                    questionId: verdict.questionId,
                    questionLabel: verdict.label,
                });
            }
        }
    }
    return unanswered;
}

export interface ReportMeta {
    readonly label: string;
    readonly modelTag: string;
}

/** The markdown report of a run (written next to the captures, outside the repo). */
export function renderReport(records: readonly ExamRecord[], meta: ReportMeta): string {
    const findings = collectFindings(records);
    const unanswered = collectUnanswered(records);
    const errors = records.filter((record) => record.error !== null);
    const overflow = records.filter((record) => record.contextOverflow);
    const screensExamined = records.length;
    const verdictsTotal = records.reduce((sum, record) => sum + record.verdicts.length, 0);

    const lines: string[] = [
        `# Экзамен экранов: ${meta.label}`,
        "",
        `Модель: \`${meta.modelTag}\` · экранов экзаменовано: ${screensExamined} · вердиктов: ${verdictsTotal} · находок: ${findings.length}`,
        "",
        "Статичные визуальные дефекты (вёрстка, глифы, шрифты); поведение и флоу — вне охвата этого прогона.",
        "Каждая находка требует ручного триажа: вердикт — вероятность, а не приговор.",
        "",
    ];

    if (findings.length > 0) {
        lines.push("| Экран | Вьюпорт | Дефект | p(true) | Порог | Скрин |", "| --- | --- | --- | --- | --- | --- |");
        for (const finding of findings) {
            const overflowMark = finding.contextOverflow ? " ⚠ ctx" : "";
            lines.push(
                `| ${finding.scenario}/${finding.screen} | ${finding.viewport} | ${finding.questionLabel}${overflowMark} | ${finding.pTrue.toFixed(3)} | ${finding.threshold} | \`${finding.imageFile}\` |`,
            );
        }
        lines.push("");
    }

    if (unanswered.length > 0) {
        lines.push("## Вопросы без ответа", "", "Модель не ответила — проверка не состоялась, «нет дефекта» здесь не доказано:", "");
        for (const item of unanswered) {
            lines.push(`- \`${item.scenario}/${item.screen} [${item.viewport}]\`: ${item.questionLabel}`);
        }
        lines.push("");
    }

    if (errors.length > 0) {
        lines.push("## Ошибки экзамена", "");
        for (const record of errors) {
            lines.push(
                `- \`${record.screen.scenario}/${record.screen.screen} [${record.screen.viewport}]\`: ${record.error}`,
            );
        }
        lines.push("");
    }

    if (overflow.length > 0) {
        lines.push(
            "## ⚠ Контекст модели близок к лимиту",
            "",
            "Промпт превысил 90% окна — вердикты этих экранов могли быть обрезаны:",
            "",
        );
        for (const record of overflow) {
            lines.push(
                `- \`${record.screen.scenario}/${record.screen.screen} [${record.screen.viewport}]\` — ${record.usage?.inputTokens ?? "?"} токенов из ${record.examiner.numCtx}`,
            );
        }
        lines.push("");
    }

    if (findings.length === 0 && errors.length === 0 && unanswered.length === 0) {
        lines.push("Дефектов не найдено, ошибок нет.");
    }

    return `${lines.join("\n")}\n`;
}

export interface IssueDraft {
    readonly title: string;
    readonly body: string;
    readonly fingerprint: string;
}

/**
 * The GitHub issue draft of a finding.
 *
 * `evidenceUrl` is the CDN link of the uploaded capture; without an
 * upload the caller must not file issues (the CLI enforces it).
 */
export function renderIssueDraft(finding: Finding, evidenceUrl: string, label: string): IssueDraft {
    const title = `[manual-test] ${finding.scenario}/${finding.screen} (${finding.viewport}): ${finding.questionLabel}`;
    const probability =
        finding.pExpected !== null
            ? `d3 выбрала «${finding.questionId}»: p выигрыша = ${finding.pTrue.toFixed(3)}, p ожидаемой = ${finding.pExpected.toFixed(3)}`
            : `d3, p(true) = ${finding.pTrue.toFixed(3)} при пороге ${finding.threshold}`;
    const body = [
        `Автоматический предрелизный прогон \`${label}\` нашёл вероятный дефект (${probability}).`,
        "",
        `- Экран: \`${finding.scenario}/${finding.screen}\` [${finding.viewport}, ${finding.locale}]`,
        `- Вопрос чеклиста: \`${finding.questionId}\` — ${finding.questionLabel}`,
        `- Скрин: ${evidenceUrl}`,
        `- sha256 скрина: \`${finding.imageSha256}\``,
        `- Снято: ${finding.capturedAt}`,
        "",
        "Вердикт — вероятность, а не приговор: подтверди дефект глазами по скрину выше.",
        "",
        "Воспроизвести прогон:",
        "",
        "```bash",
        `cd end2end && npm run shots -- capture --scenario ${finding.scenario} --label ${label} --examine --upload`,
        "```",
        "",
        `fingerprint: ${finding.fingerprint}`,
    ].join("\n");
    return { title, body, fingerprint: finding.fingerprint };
}
