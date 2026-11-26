import type {
  CoverageIssue,
  CoverageRow,
  StaticCheckInput,
} from "./coverage-types.ts";

const ROUTE_PATTERN = /path!\(\s*"([^"]+)"\s*\)/g;

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

function checkIds(rows: readonly CoverageRow[]): CoverageIssue[] {
  const seen = new Set<string>();
  const issues: CoverageIssue[] = [];
  for (const row of rows) {
    if (seen.has(row.id)) {
      issues.push({ row: row.id, problem: "duplicate inventory id" });
    }
    seen.add(row.id);
  }
  return issues;
}

function checkRoutes(input: StaticCheckInput): CoverageIssue[] {
  const routes = new Set(
    input.rows
      .map((row) => row.route)
      .filter((route): route is string => !!route),
  );
  const issues = input.routePatterns
    .filter((route) => !routes.has(route))
    .map((route) => ({ row: route, problem: "route has no inventory row" }));
  for (const alias of input.rows.map((row) => row.aliasOf).filter(Boolean)) {
    if (!routes.has(alias as string)) {
      issues.push({
        row: alias as string,
        problem: "alias target has no inventory row",
      });
    }
  }
  return issues;
}

function checkScenarios(input: StaticCheckInput): CoverageIssue[] {
  const referenced = new Set(
    input.rows.map((row) => row.scenario).filter((id): id is string => !!id),
  );
  return input.scenarioIds
    .filter((id) => !referenced.has(id))
    .map((id) => ({
      row: id,
      problem: "catalog scenario has no inventory row",
    }));
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

function checkRow(row: CoverageRow, input: StaticCheckInput): CoverageIssue[] {
  if (row.status === "excluded") {
    return row.exclusion?.code && row.exclusion.reason
      ? []
      : [{ row: row.id, problem: "exclusion needs code and reason" }];
  }
  const narrow =
    row.viewports.includes("mobile") !== row.viewports.includes("desktop");
  if (narrow && !row.viewportsReason) {
    return [{ row: row.id, problem: "single viewport needs viewportsReason" }];
  }
  if (!row.scenario) {
    return row.status === "covered"
      ? [{ row: row.id, problem: "covered row has no scenario" }]
      : [];
  }
  if (row.status === "gap") return [];
  const issues: CoverageIssue[] = [];
  if (!input.scenarioFiles.includes(`${row.scenario}.ts`)) {
    issues.push({
      row: row.id,
      problem: `missing scenario file ${row.scenario}.ts`,
    });
  }
  if (!hasOwnExam(input.examKeys, row.scenario, row.screens)) {
    issues.push({ row: row.id, problem: "no examined screen for this row" });
  }
  return issues;
}

/** Structural invariants, independent of a browser run. */
export function checkStatic(input: StaticCheckInput): CoverageIssue[] {
  return [
    ...checkIds(input.rows),
    ...checkRoutes(input),
    ...checkScenarios(input),
    ...input.rows.flatMap((row) => checkRow(row, input)),
  ];
}
