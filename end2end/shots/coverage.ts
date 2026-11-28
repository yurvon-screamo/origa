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
export type { ProofContext } from "./coverage-report.ts";
export {
  compareManifest,
  formatCoverageReport,
  provedRows,
  resolveProofFile,
} from "./coverage-report.ts";
