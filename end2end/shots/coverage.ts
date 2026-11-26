export type {
  CoverageIssue,
  CoverageKind,
  CoverageRow,
  CoverageStatus,
  ExclusionCode,
  StaticCheckInput,
} from "./coverage-types.ts";
export {
  checkStatic,
  matchesScreen,
  parseRoutePatterns,
} from "./coverage-checks.ts";
export {
  compareManifest,
  formatCoverageReport,
  provedRows,
} from "./coverage-report.ts";
