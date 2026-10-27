/**
 * The d3 decision-model client (System One API of a local Ollama).
 *
 * d3 does not chat: it answers typed questions about a state (text) and
 * attached images with a probability per question. The contract below is
 * pinned by live probes against Ollama 0.40.10 (`d3:2b`):
 *
 *   POST /v1/systemone
 *   { "model": "d3:2b", "state": "...", "images": ["<base64 png>"],
 *     "options": { "num_ctx": 16384 },
 *     "questions": { "<id>": { "type": "noul", "instructions": "...",
 *                              "criteria": { "true": "...", "false": "..." } } } }
 *   → { "model": "...", "answers": { "<id>": { "type": "noul", "noul": 0.99 } },
 *       "usage": { "input_tokens": N, "output_tokens": 0 } }
 *
 * `images` is an array of raw base64 strings (no data: prefix, no object
 * wrappers — both are rejected by the endpoint). Without an explicit
 * `num_ctx` the Modelfile default (4096) silently truncates a desktop
 * screenshot prompt (a measured 1280×800 capture costs >11k vision
 * tokens), so every call must pass a window wide enough and assert the
 * measured usage against it.
 */

/** A yes/no question: d3 returns the probability that the answer is true. */
export interface NoulQuestionSpec {
    readonly kind: "noul";
    readonly instructions: string;
    readonly criteria: { readonly true: string; readonly false: string };
}

/** A choice question: d3 scores every option, one wins. */
export interface ChoiceQuestionSpec {
    readonly kind: "choice";
    readonly instructions: string;
    /** Option id → when this option is the right pick. */
    readonly criteria: Readonly<Record<string, string>>;
}

export type QuestionSpec = NoulQuestionSpec | ChoiceQuestionSpec;

/** Token accounting of one request, as measured by Ollama. */
export interface D3Usage {
    readonly inputTokens: number;
    readonly outputTokens: number;
}

/** The answer of one question: a probability (noul) or a scored pick (choice). */
export type D3Answer =
    | { readonly kind: "noul"; readonly pTrue: number }
    | { readonly kind: "choice"; readonly choiceId: string; readonly probabilities: Readonly<Record<string, number>> };

export interface AskD3Options {
    /** Ollama base URL, e.g. `http://localhost:11434`. */
    readonly ollamaUrl: string;
    /** Model tag, e.g. `d3:2b`. */
    readonly model: string;
    readonly state: string;
    /** Base64-encoded PNG screenshots (no data: prefix). */
    readonly images?: readonly string[];
    /** Question id → question spec (noul or choice). Sent as one batched request. */
    readonly questions: ReadonlyMap<string, QuestionSpec>;
    /** Context window the request runs in; guards against silent truncation. */
    readonly numCtx: number;
    readonly timeoutMs?: number;
    /** Injectable for unit tests; defaults to the global fetch. */
    readonly fetchImpl?: typeof fetch;
}

export interface AskD3Result {
    /** Question id → probability. Questions the model left unanswered are absent. */
    readonly answers: ReadonlyMap<string, D3Answer>;
    readonly usage: D3Usage | null;
}

export class D3Error extends Error {
    constructor(message: string) {
        super(message);
        this.name = "D3Error";
    }
}

interface SystemOneResponse {
    model?: string;
    error?: string;
    answers?: Record<string, { type?: string; noul?: number; choice?: string; probabilities?: Record<string, number> }>;
    usage?: { input_tokens?: number; output_tokens?: number };
}

/** Ask every question in one batched System One request. */
export async function askD3(options: AskD3Options): Promise<AskD3Result> {
    const questions: Record<string, unknown> = {};
    for (const [id, question] of options.questions) {
        if (question.kind === "noul") {
            questions[id] = {
                type: "noul",
                instructions: question.instructions,
                criteria: { true: question.criteria.true, false: question.criteria.false },
            };
        } else {
            questions[id] = {
                type: "choice",
                instructions: question.instructions,
                criteria: question.criteria,
            };
        }
    }

    const body = JSON.stringify({
        model: options.model,
        state: options.state,
        ...(options.images && options.images.length > 0 ? { images: options.images } : {}),
        options: { num_ctx: options.numCtx },
        questions,
    });

    const fetchImpl = options.fetchImpl ?? fetch;
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), options.timeoutMs ?? 120_000);
    let response: Response;
    try {
        response = await fetchImpl(`${options.ollamaUrl}/v1/systemone`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body,
            signal: controller.signal,
        });
    } catch (error) {
        throw new D3Error(
            `Ollama request failed (${options.ollamaUrl}): ${error instanceof Error ? error.message : String(error)}`,
        );
    } finally {
        clearTimeout(timer);
    }

    if (!response.ok) {
        const detail = await response.text().catch(() => "");
        throw new D3Error(`Ollama /v1/systemone answered ${response.status}: ${detail.slice(0, 300)}`);
    }

    const payload = (await response.json()) as SystemOneResponse;
    if (payload.error) {
        throw new D3Error(`d3 rejected the request: ${payload.error}`);
    }

    const answers = new Map<string, D3Answer>();
    for (const [id, answer] of Object.entries(payload.answers ?? {})) {
        if (answer?.type === "noul" && typeof answer.noul === "number" && Number.isFinite(answer.noul)) {
            answers.set(id, { kind: "noul", pTrue: answer.noul });
        } else if (answer?.type === "choice" && typeof answer.choice === "string") {
            const probabilities = Object.fromEntries(
                Object.entries(answer.probabilities ?? {}).filter(([, p]) => Number.isFinite(p)),
            );
            answers.set(id, { kind: "choice", choiceId: answer.choice, probabilities });
        }
    }

    const usage = payload.usage
        ? {
              inputTokens: payload.usage.input_tokens ?? 0,
              outputTokens: payload.usage.output_tokens ?? 0,
          }
        : null;

    return { answers, usage };
}

/** Model tags the local Ollama instance has pulled (`GET /api/tags`). */
export async function listOllamaModels(
    ollamaUrl: string,
    fetchImpl: typeof fetch = fetch,
    timeoutMs = 3_000,
): Promise<string[]> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);
    try {
        const response = await fetchImpl(`${ollamaUrl}/api/tags`, { signal: controller.signal });
        if (!response.ok) {
            throw new D3Error(`Ollama /api/tags answered ${response.status}`);
        }
        const payload = (await response.json()) as { models?: { name?: string }[] };
        return (payload.models ?? [])
            .map((entry) => entry.name ?? "")
            .filter((name) => name.length > 0);
    } finally {
        clearTimeout(timer);
    }
}
