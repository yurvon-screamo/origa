#!/usr/bin/env node
/**
 * #749 final seed: создаёт domain_user запись для insights4@origa.local
 * с сидированным knowledge_set (по образцу существующих записей), затем
 * пишет .stitch/storage-state.json для `stitch capture --storage`.
 * Юзер insights4 уже существует (создан admin API ранее).
 */
import zlib from "node:zlib";
import { readFileSync, writeFileSync } from "node:fs";

const env = Object.fromEntries(
    readFileSync(new URL("../.env", import.meta.url), "utf8")
        .split("\n")
        .filter((l) => l.includes("="))
        .map((l) => [l.slice(0, l.indexOf("=")).trim(), l.slice(l.indexOf("=") + 1).trim()]),
);
const BASE = process.env.TRAILBASE_URL || env.TRAILBASE_URL;
const EMAIL = "insights4@origa.local";
const PASSWORD = "e2e-test-password-123";

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
const day = 86_400_000;
const now = Date.now();
const iso = (ms) => new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");

const studyCards = {};
words.forEach(([word, gloss], index) => {
    const ulid = `01J${String(index + 1).padStart(5, "0")}${"A".repeat(18)}`;
    const troubled = index < 2;
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
    const p = 8 + ((days * 3) % 4);
    const n = 2 + (days % 3);
    lessonHistory.push({
        timestamp: iso(now - days * day),
        avg_stability: 12.4,
        avg_difficulty: 4.6,
        total_words: words.length,
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
const wire = "DEFLATE;" + zlib.deflateSync(JSON.stringify(knowledgeSet)).toString("base64");

// Логин юзером (его токен создаёт свою domain_user запись).
const userLoginResponse = await fetch(`${BASE}/api/auth/v1/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email: EMAIL, password: PASSWORD }),
});
if (!userLoginResponse.ok) throw new Error(`user login: ${userLoginResponse.status}`);
const userAuth = await userLoginResponse.json();
const userHeaders = {
    "Content-Type": "application/json",
    Authorization: `Bearer ${userAuth.auth_token}`,
    "csrf-token": userAuth.csrf_token,
};

const domainBody = {
    email: EMAIL,
    native_language: 0,
    jlpt_progress: "{}",
    current_japanese_level: 3,
    knowledge_set: wire,
    imported_sets: JSON.stringify(["__onboarding_completed__"]),
    updated_at: new Date().toISOString(),
    daily_load: 1,
    known_vocab_hash: 0,
};
const upsert = await fetch(`${BASE}/api/records/v1/domain_user`, {
    method: "POST",
    headers: userHeaders,
    body: JSON.stringify(domainBody),
});
console.log("domain_user POST:", upsert.status);
if (!upsert.ok) console.log(await upsert.text());

// Storage state: сессия юзера + RU-кука.
const claims = JSON.parse(
    Buffer.from(userAuth.auth_token.split(".")[1], "base64").toString("utf8"),
);
const trailbaseId = Buffer.from(claims.sub, "base64").toString("hex")
    .replace(/^(.{8})(.{4})(.{4})(.{4})(.{12})$/, "$1-$2-$3-$4-$5");

const session = {
    auth_token: userAuth.auth_token,
    refresh_token: userAuth.refresh_token,
    email: EMAIL,
    trailbase_id: trailbaseId,
    record_id: null,
    expires_at: claims.exp,
};
const storageState = {
    cookies: [
        {
            name: "i18n_pref_locale",
            value: "ru",
            domain: "localhost",
            path: "/",
            expires: Math.floor(Date.now() / 1000) + 31_536_000,
            httpOnly: false,
            secure: false,
            sameSite: "Lax",
        },
    ],
    origins: [
        {
            origin: "http://localhost:1420",
            localStorage: [
                { name: "origa_resource_download_consented", value: "true" },
                { name: "i18n_pref_locale", value: "ru" },
                { name: "trailbase_session", value: JSON.stringify(session) },
            ],
        },
    ],
};

writeFileSync(".stitch/storage-state.json", JSON.stringify(storageState, null, 2));
console.log("storage state written; sub:", claims.sub);
