import zlib from "node:zlib";
import { readFileSync } from "node:fs";

const env = Object.fromEntries(
    readFileSync(new URL("../.env", import.meta.url), "utf8")
        .split("\n")
        .filter((l) => l.includes("="))
        .map((l) => [l.slice(0, l.indexOf("=")).trim(), l.slice(l.indexOf("=") + 1).trim()]),
);
const BASE = process.env.TRAILBASE_URL || env.TRAILBASE_URL;

// Юзер патчит СВОЮ domain_user запись (создана приложением при логине).
const admin = await fetch(`${BASE}/api/auth/v1/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email: "insights2@origa.local", password: "e2e-test-password-123" }),
});
const { auth_token, csrf_token } = await admin.json();
const headers = {
    "Content-Type": "application/json",
    Authorization: `Bearer ${auth_token}`,
    "csrf-token": csrf_token,
};

const seed = JSON.parse(readFileSync("/tmp/opencode/seed-ks.json", "utf8"));
// Тот же формат, что у приложения: сид + completed-флаг онбординга.
seed.lesson_history = [];
const day = 86_400_000;
const now = Date.now();
const iso = (ms) => new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");
for (const days of [2, 4, 6, 8, 10, 12, 15, 18, 21, 24, 27]) {
    const p = 8 + ((days * 3) % 4);
    const n = 2 + (days % 3);
    seed.lesson_history.push({
        timestamp: iso(now - days * day),
        avg_stability: 12.4,
        avg_difficulty: 4.6,
        total_words: 24,
        new_words: 2,
        known_words: 10,
        in_progress_words: 10,
        high_difficulty_words: 2,
        new_cards_studied_today: 2,
        phrase_cards_studied_today: 1,
        lessons_completed: 1,
        positive_ratings: p,
        negative_ratings: n,
        total_ratings: p + n,
    });
}

const wire = "DEFLATE;" + zlib.deflateSync(JSON.stringify(seed)).toString("base64");
const patch = await fetch(`${BASE}/api/records/v1/domain_user/2`, {
    method: "PATCH",
    headers,
    body: JSON.stringify({
        knowledge_set: wire,
        updated_at: new Date().toISOString(),
    }),
});
console.log("patch /2:", patch.status);
