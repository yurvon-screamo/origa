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

export function formatCoverageReport(
  rows: readonly CoverageRow[],
  issues: readonly CoverageIssue[],
): string {
  const gaps = rows.filter((row) => row.status === "gap");
  const excluded = rows.filter((row) => row.status === "excluded");
  const lines = [
    `Инвентарь: ${rows.length} строк; покрыто ${rows.length - gaps.length - excluded.length}, дыр ${gaps.length}, исключений ${excluded.length}.`,
  ];
  if (gaps.length > 0) {
    lines.push("", "Дыры:");
    for (const row of gaps) lines.push(`  ${row.id} — ${row.source}`);
  }
  if (issues.length > 0) {
    lines.push("", "Проблемы:");
    for (const issue of issues) lines.push(`  ${issue.row}: ${issue.problem}`);
  } else {
    lines.push("", "Структурных проблем нет.");
  }
  return `${lines.join("\n")}\n`;
}
