#!/usr/bin/env node
/**
 * Writes the seeded typical-user knowledge_set into the existing
 * domain_user record of insights3@origa.local (#749 Stitch proposals).
 * The record must already exist (created by the app on first login).
 */
import zlib from "node:zlib";
import { readFileSync } from "node:fs";

const env = Object.fromEntries(
    readFileSync(new URL("../.env", import.meta.url), "utf8")
        .split("\n")
        .filter((line) => line.includes("="))
        .map((line) => [line.slice(0, line.indexOf("=")).trim(), line.slice(line.indexOf("=") + 1).trim()]),
);
const BASE = process.env.TRAILBASE_URL || env.TRAILBASE_URL;
const EMAIL = "insights2@origa.local";
const PASSWORD = "e2e-test-password-123";

const login = await fetch(`${BASE}/api/auth/v1/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email: EMAIL, password: PASSWORD }),
});
if (!login.ok) throw new Error(`demo login failed: ${login.status}`);
const { auth_token, csrf_token } = await login.json();
const headers = {
    "Content-Type": "application/json",
    Authorization: `Bearer ${auth_token}`,
    "csrf-token": csrf_token,
};

const find = await fetch(
    `${BASE}/api/records/v1/domain_user?filter[email][$eq]=${encodeURIComponent(EMAIL)}`,
    { headers },
);
if (!find.ok) throw new Error(`record fetch failed: ${find.status}`);
const found = await find.json();
const record = (found.records || [])[0];
if (!record) throw new Error("domain_user record not found — log in via the app once");
const recordId = record.id;
// На всякий случай помечаем онбординг завершённым: иначе app уведёт
// capture на /onboarding.
const onboarded =
    typeof record.imported_sets === "string" && record.imported_sets.includes("__onboarding")
        ? record.imported_sets
        : JSON.stringify(["__onboarding_completed__"]);

const day = 86_400_000;
const now = Date.now();
const iso = (ms) => new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");
const words = [
    ["結局", "in the end"], ["つまり", "in short"], ["それでも", "even so"],
    ["たとえば", "for example"], ["すなわち", "namely"], ["おそらく", "probably"],
    ["どうやって", "how"], ["やはり", "as expected"], ["もう一度", "once more"],
    ["できるだけ", "as much as possible"], ["やっぱり", "still"], ["けっこう", "considerably"],
    ["びっくり", "surprise"], ["ゆっくり", "slowly"], ["たぶん", "probably"],
    ["ほとんど", "mostly"], ["ちゃんと", "properly"], ["やっと", "at last"],
    ["きっと", "surely"], ["あまり", "not much"], ["とても", "very"],
    ["たくさん", "a lot"], ["少し", "a little"], ["みんな", "everyone"],
];
const studyCards = {};
words.forEach(([word, gloss], index) => {
    // Валидный ULID: ровно 26 символов Crockford-base32.
    const ulid = `01J${String(index + 1).padStart(5, "0")}${"A".repeat(18)}`;
    const troubled = index < Math.floor(words.length / 12) + 1;
    const overdue = index % 2 === 0;
    const lastReview = now - (overdue ? 12 : 2) * day;
    studyCards[ulid] = {
        card_id: ulid,
        card: {
            Vocabulary: {
                word: { text: word },
                reverse_side: { text: gloss },
                pos: null,
                tokens: null,
            },
        },
        memory_history: {
            current_state: {
                stability: { value: troubled ? 4.2 : 14.8 },
                difficulty: { value: troubled ? 7.4 : 4.6 },
                next_review_date: iso(overdue ? lastReview + day : now + 5 * day),
                card_state: "Review",
            },
            reps: troubled ? 9 : 6,
            lapses: troubled ? 4 : 0,
            easy_count: troubled ? 0 : 2,
            good_count: troubled ? 3 : 4,
            last_review_date: iso(lastReview),
            last_rating: troubled ? "Again" : "Good",
            consecutive_again: 0,
            ghost: null,
            marked_known_at: null,
        },
        is_favorite: false,
        perfect_streak_since_known: 0,
        favorite_changed_at: null,
    };
});
const lessonHistory = [];
for (const days of [2, 4, 6, 8, 10, 12, 15, 18, 21, 24, 27]) {
    const dayPositive = 8 + ((days * 3) % 4);
    const dayNegative = 2 + (days % 3);
    lessonHistory.push({
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
        positive_ratings: dayPositive,
        negative_ratings: dayNegative,
        total_ratings: dayPositive + dayNegative,
    });
}
const knowledgeSet = {
    study_cards: studyCards,
    deleted_cards: [],
    deleted_companion_words: [],
    lesson_history: lessonHistory,
    insights: {
        hour_attempts: [2, 4, 8, 14, 30, 26],
        hour_correct: [2, 4, 7, 12, 22, 13],
        delay_attempts: [60, 20, 8, 4],
        delay_correct: [52, 13, 4, 1],
    },
};
const wire = "DEFLATE;" + zlib.deflateRawSync(JSON.stringify(knowledgeSet)).toString("base64");

const patch = await fetch(`${BASE}/api/records/v1/domain_user/${recordId}`, {
    method: "PATCH",
    headers,
    body: JSON.stringify({
        knowledge_set: wire,
        imported_sets: onboarded,
        updated_at: new Date().toISOString(),
    }),
});
console.log("patch status:", patch.status, "| record:", recordId);
if (!patch.ok) console.log(await patch.text());
