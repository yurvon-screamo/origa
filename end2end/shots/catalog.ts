import type { PresetId } from "./presets/registry.ts";
import { DEFAULT_VIEWPORTS, type ViewportTag } from "./viewports.ts";

/**
 * Everything about a screen except HOW to reach it.
 *
 * The catalog holds the metadata only, so the CLI can list the screens
 * without loading the browser-side modules. The implementations in
 * `shots/scenarios/` spread these descriptors and add the `run` step —
 * one source of truth for the metadata.
 */
export interface ScenarioDescriptor {
    /** Stable identifier — the registry key and the CLI argument. */
    id: string;
    /** Human title (Russian) for the registry and the reports. */
    title: string;
    /** Route the screen belongs to. */
    route: string;
    locale: "ru-RU" | "en-US";
    /** Viewports to capture; the desktop/phone pair by default. */
    viewports?: ViewportTag[];
    /** Alternative build to capture (a before/after pair on another port). */
    baseURL?: string;
    /** Data state the screen is shown in. */
    preset: PresetId;
}

export const descriptors: Record<string, ScenarioDescriptor> = {
    home: {
        id: "home",
        title: "Главный экран",
        route: "/home",
        locale: "ru-RU",
        viewports: DEFAULT_VIEWPORTS,
        preset: "fresh",
    },
};

export function listDescriptors(): ScenarioDescriptor[] {
    return Object.values(descriptors).sort((a, b) => a.id.localeCompare(b.id));
}
