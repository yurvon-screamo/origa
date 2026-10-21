/**
 * Viewports of the screenshot tool.
 *
 * The two sizes are the ones the factory pipeline asks every UI pull
 * request for: a desktop review view and a phone view.
 */
export const VIEWPORTS = {
    desktop: { tag: "desktop", width: 1280, height: 800 },
    mobile: { tag: "mobile", width: 390, height: 844 },
} as const;

export type ViewportTag = keyof typeof VIEWPORTS;

export const DEFAULT_VIEWPORTS: ViewportTag[] = ["desktop", "mobile"];
