import type { ShotEntry } from "./manifest.ts";
import type { ViewportTag } from "./viewports.ts";

export type CoverageKind = "page" | "variation" | "surface";
export type CoverageStatus = "covered" | "gap" | "excluded";
export type ExclusionCode =
  | "animation"
  | "loading"
  | "error"
  | "tauri-only"
  | "feature-off"
  | "owner-question";

/** One obligation: this screen must be captured for this scenario. */
export interface CoverageRow {
  readonly id: string;
  readonly kind: CoverageKind;
  /** Route pattern from routes.rs, or null for a screen outside the router. */
  readonly route: string | null;
  /** Stable symbol or test-id — never a line number. */
  readonly source: string;
  readonly scenario: string | null;
  /** sink.shot() names; a trailing * matches a dynamic suffix. */
  readonly screens: readonly string[];
  readonly viewports: readonly ViewportTag[];
  /** Required when the row narrows the ADR-073 pair to one viewport. */
  readonly viewportsReason?: string;
  readonly status: CoverageStatus;
  readonly exclusion?: {
    readonly code: ExclusionCode;
    readonly reason: string;
    readonly source?: string;
  };
  /** Set when the route is an alias of another route. */
  readonly aliasOf?: string;
}

export interface CoverageIssue {
  readonly row: string;
  readonly problem: string;
}

export interface StaticCheckInput {
  readonly rows: readonly CoverageRow[];
  readonly routePatterns: readonly string[];
  readonly scenarioIds: readonly string[];
  readonly scenarioFiles: readonly string[];
  readonly examKeys: readonly string[];
}

export const ROUTE_PATTERN = /path!\(\s*"([^"]+)"\s*\)/g;

/** Reads the only router table of the web app from its Leptos source. */
export function parseRoutePatterns(source: string): string[] {
  const patterns = new Set<string>();
  for (const match of source.matchAll(ROUTE_PATTERN)) {
    const path = match[1];
    patterns.add(path.startsWith("/") ? path : `/${path}`);
  }
  return [...patterns].sort();
}

export function matchesScreen(pattern: string, screen: string): boolean {
  if (pattern.endsWith("*")) return screen.startsWith(pattern.slice(0, -1));
  return pattern === screen;
}

function hasOwnExam(
  examKeys: readonly string[],
  scenario: string,
  screens: readonly string[],
): boolean {
  const keys = examKeys
    .filter((key) => key.startsWith(`${scenario}:`))
    .map((key) => key.slice(scenario.length + 1));
  return screens.some((pattern) =>
    keys.some((key) => matchesScreen(pattern, key)),
  );
}

/** Structural invariants, independent of a browser run. */
export function checkStatic(input: StaticCheckInput): CoverageIssue[] {
  const issues: CoverageIssue[] = [];
  const rows = new Map(input.rows.map((row) => [row.id, row]));
  for (const [id, row] of rows) {
    if (id !== row.id) issues.push({ row: id, problem: "id does not match" });
  }

  const coveredRoutes = new Set(
    input.rows
      .map((row) => row.route)
      .filter((route): route is string => !!route),
  );
  for (const route of input.routePatterns) {
    if (!coveredRoutes.has(route)) {
      issues.push({ row: route, problem: "route has no inventory row" });
    }
  }
  const aliased = new Set(
    input.rows
      .map((row) => row.aliasOf)
      .filter((alias): alias is string => !!alias),
  );
  for (const alias of aliased) {
    if (!coveredRoutes.has(alias)) {
      issues.push({ row: alias, problem: "alias target has no inventory row" });
    }
  }

  const referenced = new Set(
    input.rows.map((row) => row.scenario).filter((id): id is string => !!id),
  );
  const files = new Set(input.scenarioFiles);
  for (const id of input.scenarioIds) {
    if (!referenced.has(id)) {
      issues.push({
        row: id,
        problem: "catalog scenario has no inventory row",
      });
    }
  }

  for (const row of input.rows) {
    if (row.status === "excluded") {
      if (!row.exclusion?.code || !row.exclusion.reason) {
        issues.push({
          row: row.id,
          problem: "exclusion needs code and reason",
        });
      }
      continue;
    }
    if (
      row.viewports.includes("mobile") !== row.viewports.includes("desktop")
    ) {
      if (!row.viewportsReason) {
        issues.push({
          row: row.id,
          problem: "single viewport needs viewportsReason",
        });
      }
    }
    if (!row.scenario) {
      if (row.status === "covered") {
        issues.push({ row: row.id, problem: "covered row has no scenario" });
      }
      continue;
    }
    if (row.status === "gap") continue;
    if (!files.has(`${row.scenario}.ts`)) {
      issues.push({
        row: row.id,
        problem: `missing scenario file ${row.scenario}.ts`,
      });
    }
    if (!hasOwnExam(input.examKeys, row.scenario, row.screens)) {
      issues.push({ row: row.id, problem: "no examined screen for this row" });
    }
  }
  return issues;
}

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
        const found = entries.some(
          (entry) =>
            entry.scenario === row.scenario &&
            entry.viewport === viewport &&
            matchesScreen(screen, entry.screen),
        );
        if (!found) {
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
    for (const row of gaps) {
      lines.push(`  ${row.id} — ${row.source}`);
    }
  }
  if (issues.length > 0) {
    lines.push("", "Проблемы:");
    for (const issue of issues) {
      lines.push(`  ${issue.row}: ${issue.problem}`);
    }
  } else {
    lines.push("", "Структурных проблем нет.");
  }
  return `${lines.join("\n")}\n`;
}
