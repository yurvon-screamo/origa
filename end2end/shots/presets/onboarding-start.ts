import type { PresetContext } from "./index.ts";

/** Leaves the onboarding wizard untouched so its steps can be photographed. */
export async function onboardingStart(_context: PresetContext): Promise<void> {
  return;
}
