import assert from "node:assert/strict";
import { test } from "node:test";

import { askD3, D3Error, listOllamaModels } from "./d3.ts";

function jsonResponse(payload: unknown, status = 200): Response {
    return new Response(JSON.stringify(payload), {
        status,
        headers: { "Content-Type": "application/json" },
    });
}

test("askD3 sends one batched noul request with num_ctx and base64 images", async () => {
    let seen: Request | null = null;
    const fetchImpl: typeof fetch = async (input, init) => {
        seen = new Request(input as URL, init);
        return jsonResponse({
            model: "d3:2b",
            answers: { overlap: { type: "noul", noul: 0.9 } },
            usage: { input_tokens: 1500, output_tokens: 0 },
        });
    };

    const result = await askD3({
        ollamaUrl: "http://localhost:11434",
        model: "d3:2b",
        state: "state text",
        images: ["aGk="],
        questions: new Map([
            ["overlap", { kind: "noul", instructions: "i", criteria: { true: "t", false: "f" } }],
            ["clipped", { kind: "noul", instructions: "i", criteria: { true: "t", false: "f" } }],
        ]),
        numCtx: 16_384,
        fetchImpl,
    });

    const body = JSON.parse((seen!.body ? await new Request(seen!).text() : "") || "{}");
    assert.equal(body.model, "d3:2b");
    assert.equal(body.state, "state text");
    assert.deepEqual(body.images, ["aGk="]);
    assert.deepEqual(body.options, { num_ctx: 16384 });
    assert.equal(body.questions.overlap.type, "noul");
    assert.equal(body.questions.overlap.criteria.true, "t");
    // Two questions were asked in one request.
    assert.equal(Object.keys(body.questions).length, 2);

    const overlap = result.answers.get("overlap");
    assert.equal(overlap?.kind === "noul" && overlap.pTrue, 0.9);
    // The model left the second question unanswered — it is simply absent.
    assert.equal(result.answers.has("clipped"), false);
    assert.equal(result.usage?.inputTokens, 1500);
});

test("askD3 omits images when none are attached", async () => {
    let raw = "";
    const fetchImpl: typeof fetch = async (input, init) => {
        raw = String((init?.body as string) ?? "");
        return jsonResponse({ model: "d3:2b", answers: {}, usage: null });
    };
    await askD3({
        ollamaUrl: "http://x",
        model: "d3:2b",
        state: "s",
        questions: new Map([["q", { kind: "noul", instructions: "i", criteria: { true: "t", false: "f" } }]]),
        numCtx: 4096,
        fetchImpl,
    });
    assert.equal("images" in JSON.parse(raw), false);
});

test("askD3 turns a non-2xx response into a D3Error with the body detail", async () => {
    const fetchImpl: typeof fetch = async () => jsonResponse({ error: "bad model" }, 400);
    await assert.rejects(
        askD3({
            ollamaUrl: "http://x",
            model: "nope",
            state: "s",
            questions: new Map([["q", { kind: "noul", instructions: "i", criteria: { true: "t", false: "f" } }]]),
            numCtx: 4096,
            fetchImpl,
        }),
        (error: unknown) => error instanceof D3Error && error.message.includes("400"),
    );
});

test("askD3 reports a network failure as a D3Error", async () => {
    const fetchImpl: typeof fetch = async () => {
        throw new Error("connection refused");
    };
    await assert.rejects(
        askD3({
            ollamaUrl: "http://x",
            model: "d3:2b",
            state: "s",
            questions: new Map([["q", { kind: "noul", instructions: "i", criteria: { true: "t", false: "f" } }]]),
            numCtx: 4096,
            fetchImpl,
        }),
        D3Error,
    );
});

test("askD3 ignores malformed noul values instead of crashing", async () => {
    const fetchImpl: typeof fetch = async () =>
        jsonResponse({
            answers: {
                good: { type: "noul", noul: 0.5 },
                bad: { type: "noul", noul: "yes" },
                worse: { type: "noul" },
                stray: { type: "choice", choice: "x", probabilities: { x: 1 } },
            },
        });
    const result = await askD3({
        ollamaUrl: "http://x",
        model: "d3:2b",
        state: "s",
        questions: new Map([["good", { kind: "noul", instructions: "i", criteria: { true: "t", false: "f" } }]]),
        numCtx: 4096,
        fetchImpl,
    });
    // The parser keeps every well-formed answer (the verdict mapping
    // iterates the questions, so a stray answer cannot sneak in); the
    // malformed ones are dropped.
    assert.equal(result.answers.size, 2);
    const good = result.answers.get("good");
    assert.equal(good?.kind === "noul" && good.pTrue, 0.5);
    const stray = result.answers.get("stray");
    assert.equal(stray?.kind === "choice" && stray.choiceId, "x");
});

test("listOllamaModels returns the pulled tags", async () => {
    const fetchImpl: typeof fetch = async () =>
        jsonResponse({ models: [{ name: "d3:2b" }, { name: "d3:4b" }, {}] });
    const models = await listOllamaModels("http://x", fetchImpl);
    assert.deepEqual(models, ["d3:2b", "d3:4b"]);
});

test("listOllamaModels surfaces a failure", async () => {
    const fetchImpl: typeof fetch = async () => jsonResponse({}, 500);
    await assert.rejects(listOllamaModels("http://x", fetchImpl), D3Error);
});
