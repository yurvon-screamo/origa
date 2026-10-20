/**
 * Waiting primitives that express a STATE, not a duration.
 *
 * Decision tree for every wait in the suite:
 *
 *   1) The expected state is observable (a DOM node, a value in the local
 *      store) → `expect.poll` / `expect(...)`: Playwright retries until the
 *      state appears, no sleep involved.
 *   2) A counter has to SETTLE (a request stream, a rendered progress) →
 *      `waitUntilStable`: the wait ends when the counter stops changing.
 *   3) Nothing marks the end (network silence) → `waitUntilStable` with an
 *      explicit quiet window.
 *
 * A blind `waitForTimeout` is only allowed for an intentional animation
 * pause — and then it carries a comment saying why.
 */

/** Options of {@link waitUntilStable}. */
export interface StableWaitOptions {
    /** Name of the observed counter — it appears in the error message. */
    label: string;
    /** How long the counter must stay unchanged to count as settled. */
    quietMs: number;
    /** Upper bound of the whole wait. */
    timeoutMs: number;
    /** Sampling interval (default 250 ms). */
    pollMs?: number;
}

const DEFAULT_POLL_MS = 250;

function sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Waits until the observed counter stops changing for `quietMs`.
 *
 * Use it when the end of an operation is not marked by a single state but
 * by the absence of further changes: a chunk stream that stopped asking
 * for data, a request log that went quiet, a rendered list that settled.
 *
 * The wait fails loudly with the last observed value — a silent fallback
 * would move the failure into a later, less readable assertion.
 */
export async function waitUntilStable(
    read: () => number | Promise<number>,
    options: StableWaitOptions,
): Promise<void> {
    const pollMs = options.pollMs ?? DEFAULT_POLL_MS;
    const deadline = Date.now() + options.timeoutMs;
    let last = await read();
    let lastChangeAt = Date.now();

    while (Date.now() < deadline) {
        await sleep(pollMs);
        const current = await read();
        if (current !== last) {
            last = current;
            lastChangeAt = Date.now();
            continue;
        }
        if (Date.now() - lastChangeAt >= options.quietMs) {
            return;
        }
    }

    throw new Error(
        `waitUntilStable(${options.label}): the counter did not settle within ` +
            `${options.timeoutMs} ms — last value ${last}, required quiet window ` +
            `${options.quietMs} ms`,
    );
}
