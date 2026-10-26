# ADR-068: Pre-release screen examiner (d3 decision model)

## Status

Accepted (extends ADR-067)

## Date

2026-10-25

## Context

The scripted e2e suite asserts the DOM and the behavior; nobody looks at
the pixels. A broken brand font (sans-serif instead of Cormorant
Garamond), tofu glyphs in Japanese text, clipped cards and skewed layout
pass every existing check, and the screenshot tool of ADR-067 captures
them all — faithfully reproducing the defect for the pull request body.

A pre-release manual pass is exactly the kind of work that does not
scale by hand and is invisible to CI: it needs eyes on every screen of
every scenario on every viewport, comparing what is rendered against
what should be there.

The d3 family (vLLM Semantic Router, served locally by Ollama) answers
typed yes/no questions about a state and an image with a calibrated
probability — no generated prose, no chat. References (Jev-Mobile,
CUA-JEV) use such decision models inside computer-use loops; this
project needs only the smallest slice: a judge over still screenshots,
because the deterministic walk already exists (the shot scenarios).

## Decision

`capture --examine` sends every captured screen through the local d3
model (Ollama `/v1/systemone`, one batched request per screen: state
text + screenshot + all questions of the screen):

- The examiner lives in `end2end/shots/examiner/` — an extension of the
  screenshot tool of ADR-067, not a new capture path. The scenarios and
  presets are not touched; the sink calls the examiner after every
  `shot()`.
- The test model (`examiner/model.json`) is committed: which screens are
  examined and with which questions, full coverage of the catalog
  enforced by a unit test (a new scenario without an examined screen
  fails the suite). A screen key may end with `*` — a prefix match for
  dynamically named captures (`review-body-<kind>-<card>`). A screen
  outside the model is skipped loudly (the CLI prints the coverage).
- The defect checklist (`examiner/questions.ts`) holds typed `noul`
  questions about static visual defects: overlap, clipping, tofu glyphs,
  misalignment, empty states, wrong typeface. Behavior, hover and flow
  are out of scope of the defect checklist — the flow side is covered by
  the semantic oracle below.
- **The semantic oracle over the lesson flow.** Beyond defect questions,
  the model asks `choice` questions — the workhorse is `lesson-stage`:
  the test model states which lesson stage a healthy screen must show
  (`expect` in model.json), and the model picks one of the named
  options with per-option probabilities. The randomization of the
  scheduler (card kinds, distractors, order) does not matter to a
  semantic question — «which stage is this?» — which is exactly the
  hole the scripted asserts cannot cover. A mismatch is a finding
  («ждали X, увидели Y»). The negative canary proves the oracle does
  not rubber-stamp: a home screen must NOT read as a quiz stage.
- The lesson coverage: the whole catalog (every screen of every
  scenario, fullPage included) plus the review body walk
  (`lesson-review-body` scenario on the `due-corpus` preset) with a
  wildcard model entry expecting one of the revealed-card stages.
- Every request passes an explicit `num_ctx`: a measured 1280×800
  screenshot costs over 11k vision tokens, which the Modelfile default
  (4096) truncates silently. The measured usage is recorded; a prompt
  above 90% of the window is a visible warning in the report.
- A finding is a probability, not a verdict. The report demands manual
  triage; `--file-issues` (opt-in, requires `--upload` for public
  evidence) files one GitHub issue per finding with a stable
  `fingerprint:` dedup line and the `ai-manual-test` label.
- Model runtime failures are recorded per screen (one retry first) and
  fail the run loudly at the end; the captures survive.
- The canary (`screenshots-spec/exam-canary.spec.ts`, `npm run
  exam:canary`) proves the examiner works: a clean screen stays
  unflagged and every injected defect is caught, two passes in a row.
  It needs a live Ollama and is not part of CI.

### Measured capabilities (the POC calibration, 2026-10-26)

The canary is the arbiter of what a question may claim. Measured on the
home screen with d3:2b and d3:4b (multiple passes each):

| Question | d3:2b | d3:4b | In the committed model |
| --- | --- | --- | --- |
| text-overlap | 0.78–0.82 caught | 0.63 missed | yes (the 2b default) |
| broken-state | 0.94 caught | caught | yes |
| tofu-glyphs | calm on clean (0.03–0.05); no canary | — | yes: specificity proven, sensitivity unproven |
| text-clipped | 0.27–0.37 missed | 0.37 missed | no — measured blind spot |
| layout-misaligned | 0.01–0.19 missed | 0.15 missed | no — measured blind spot |
| wrong-typeface | 0.61–0.71 (at the clean noise floor) | missed | no — cannot tell the faces apart (Comic Sans included) |

The POC ships three canary-proven questions; the blind spots stay in
the checklist with documented evidence and return only with a passing
canary (a better model, or better formulations). Neither model covers
the whole checklist — the d3:4b calibration session is the next step,
not a blocker.

## Consequences

- The pre-release pass gains an automated first look at the pixels; the
  human triage moves from «find the defect» to «confirm the flag».
- The examiner is only as good as its questions: the checklist is an
  iterated artifact, and the canary is the regression test of the
  examiner itself (a question that stops catching its canary is a
  broken question).
- The examiner caught a real defect during its own POC: a worktree
  bundle built before the font corpus existed cached an empty `fonts.rs`
  — every screen rendered in a fallback face while the corpus preflight
  stayed green. The class «the bundle is wrong, the checks are green»
  is exactly what a pixel-level examiner is for.
- The tool stays local and offline: an Ollama with the model pulled
  (`d3:2b` default, `d3:4b` via `--exam-model`), no data leaves the
  machine unless `--upload`/`--file-issues` explicitly publish evidence.
- Full computer-use exploration (the model choosing actions, the
  Jev-Mobile pattern) remains future work; this decision covers the
  deterministic walk only.
