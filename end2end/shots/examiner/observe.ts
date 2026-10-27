/**
 * The DOM snapshot: what d3 sees of the page besides the pixels.
 *
 * Same idea as the accessibility tree of the Jev-Mobile reference: a
 * compact textual view of the visible elements, so the decision model
 * can cross-check what the screenshot shows against what the page
 * claims to be on screen. The snapshot is capped hard — the System One
 * request shares its context window with the vision tokens of the
 * screenshot and the batched questions.
 */
export interface ObservedNode {
    readonly tag: string;
    readonly testid: string | null;
    readonly role: string | null;
    readonly text: string;
}

/** Character budget for the formatted snapshot (measured against num_ctx). */
export const DOM_SNAPSHOT_LIMIT = 2400;

/**
 * Runs inside the page (page.evaluate): collects the visible marked
 * elements. Must stay self-contained — Playwright serializes the whole
 * function into the browser (no closures over module constants).
 */
export function observeVisibleNodes(): unknown[] {
    const selector = '[data-testid], h1, h2, h3, h4, button, a, [role]';
    const nodes: unknown[] = [];
    for (const element of Array.from(document.querySelectorAll(selector))) {
        if ((element as Element).getClientRects().length === 0) continue;
        const text = ((element as Element).textContent || "")
            .replace(/\s+/g, " ")
            .trim()
            .slice(0, 80);
        nodes.push({
            tag: (element as Element).tagName.toLowerCase(),
            testid: (element as Element).getAttribute("data-testid"),
            role: (element as Element).getAttribute("role"),
            text,
        });
        if (nodes.length >= 150) break;
    }
    return nodes;
}

export interface DomSnapshotInput {
    readonly route: string;
    readonly locale: string;
    readonly viewport: string;
    readonly nodes: readonly ObservedNode[];
}

export interface DomSnapshot {
    readonly text: string;
    readonly nodeCount: number;
    readonly truncatedNodes: number;
}

/** Formats the observed nodes into the compact state block for d3. */
export function formatDomSnapshot(input: DomSnapshotInput, limit: number = DOM_SNAPSHOT_LIMIT): DomSnapshot {
    const header = `route=${input.route} locale=${input.locale} viewport=${input.viewport}`;
    const lines: string[] = [];
    let used = header.length;
    let truncated = 0;

    for (const node of input.nodes) {
        const parts = [node.tag];
        if (node.testid) parts.push(`#${node.testid}`);
        if (node.role) parts.push(`role=${node.role}`);
        if (node.text) parts.push(`— ${node.text}`);
        const line = parts.join(" ");
        // +1 for the newline the join below will add.
        if (used + line.length + 1 > limit) {
            truncated = input.nodes.length - lines.length;
            break;
        }
        used += line.length + 1;
        lines.push(line);
    }

    const suffix = truncated > 0 ? `\n… [${truncated} more elements truncated]` : "";
    return {
        text: `${header}\n${lines.join("\n")}${suffix}`,
        nodeCount: lines.length,
        truncatedNodes: truncated,
    };
}
