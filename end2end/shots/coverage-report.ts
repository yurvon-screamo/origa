import type { ShotEntry } from "./manifest.ts";
import { matchesScreen } from "./coverage-checks.ts";
import type { CoverageIssue, CoverageRow } from "./coverage-types.ts";

/** Matches captured frames to rows; every required viewport must be present. */
export function compareManifest(
  rows: readonly CoverageRow[],
  entries: readonly ShotEntry[],
): CoverageIssue[] {
  const issues: CoverageIssue[] = [];
  const commits = new Set(entries.map((entry) => entry.commit));
  if (commits.size > 1) {
    issues.push({
      row: "manifest",
      problem: `mixed commits in one proof: ${[...commits].sort().join(", ")}`,
    });
  }
  for (const row of rows) {
    if (row.status !== "covered" || !row.scenario) continue;
    for (const viewport of row.viewports) {
      for (const screen of row.screens) {
        if (!hasFrame(entries, row, screen, viewport)) {
          issues.push({
            row: row.id,
            problem: `no ${viewport} frame for screen "${screen}"`,
          });
        }
      }
    }
  }
  return issues;
}

function hasFrame(
  entries: readonly ShotEntry[],
  row: CoverageRow,
  screen: string,
  viewport: string,
): boolean {
  return entries.some(
    (entry) =>
      entry.scenario === row.scenario &&
      entry.viewport === viewport &&
      matchesScreen(screen, entry.screen),
  );
}

/** A proof with mixed commits is invalid as a whole, not per row. */
export function provedRows(
  rows: readonly CoverageRow[],
  issues: readonly CoverageIssue[],
): string[] {
  if (issues.some((issue) => issue.row === "manifest")) return [];
  const failing = new Set(issues.map((issue) => issue.row));
  return rows
    .filter((row) => row.status === "covered" && !failing.has(row.id))
    .map((row) => row.id);
}

export function formatCoverageReport(
  rows: readonly CoverageRow[],
  issues: readonly CoverageIssue[],
  proved: readonly string[] = [],
): string {
  const covered = rows.filter((row) => row.status === "covered");
  const gaps = rows.filter((row) => row.status === "gap");
  const excluded = rows.filter((row) => row.status === "excluded");
  const lines = [
    `Инвентарь: ${rows.length} строк; декларировано покрыто ${covered.length}, дыр ${gaps.length}, исключений ${excluded.length}; доказано манифестом ${proved.length}.`,
    "",
    "Строки:",
  ];
  for (const row of covered) {
    lines.push(
      `  covered  ${row.id} — ${row.source} [${row.viewports.join(",")}]${proved.length > 0 || issues.length > 0 ? (proved.includes(row.id) ? " · кадры есть" : " · кадров нет") : " · доказательство не проверено"}`,
    );
  }
  for (const row of gaps) {
    lines.push(
      `  gap      ${row.id} — ${row.source} [${row.viewports.join(",")}]`,
    );
  }
  for (const row of excluded) {
    lines.push(
      `  excluded ${row.id} — ${row.exclusion?.code}: ${row.exclusion?.reason}`,
    );
  }
  const questions = rows.filter(
    (row) => row.exclusion?.code === "owner-question",
  );
  if (questions.length > 0) {
    lines.push("", "Вопросы владельцу:");
    for (const row of questions) {
      lines.push(`  ${row.id}: ${row.exclusion?.reason}`);
    }
  }
  if (issues.length > 0) {
    lines.push("", `Проблемы (${issues.length}):`);
    for (const issue of issues) {
      lines.push(`  ${issue.row}: ${issue.problem}`);
    }
  } else {
    lines.push("", "Структурных проблем нет.");
  }
  return `${lines.join("\n")}\n`;
}
