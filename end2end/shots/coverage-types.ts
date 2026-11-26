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
