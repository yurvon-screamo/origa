#!/usr/bin/env node
/**
 * Seeds a demo user with a typical active-state knowledge set straight
 * into TrailBase (#749 Stitch proposals). No browser: user record via
 * admin API, knowledge_set in the DEFLATE;base64(JSON) wire format the
 * app itself writes (see origa_ui/src/repository/knowledge_set_codec.rs).
 *
 * Usage: node scripts/seed-insights-demo.mjs
 * Env: TRAILBASE_URL, ORIGA_ADMIN_EMAIL, ORIGA_ADMIN_PASSWORD
 */
import zlib from "node:zlib";
import { readFileSync } from "node:fs";

const env = Object.fromEntries(
    readFileSync(new URL("../.env", import.meta.url), "utf8")
        .split("\n")
        .filter((line) => line.includes("="))
        .map((line) => {
            const idx = line.indexOf("=");
            return [line.slice(0, idx).trim(), line.slice(idx + 1).trim()];
        }),
);
const BASE = process.env.TRAILBASE_URL || env.TRAILBASE_URL;
const ADMIN_EMAIL = process.env.ORIGA_ADMIN_EMAIL || env.ORIGA_ADMIN_EMAIL;
const ADMIN_PASSWORD = process.env.ORIGA_ADMIN_PASSWORD || env.ORIGA_ADMIN_PASSWORD;
const DEMO_EMAIL = process.env.DEMO_EMAIL || "insights2@origa.local";
const DEMO_PASSWORD = "e2e-test-password-123";

const api = async (path, options = {}) => {
    const response = await fetch(`${BASE}${path}`, options);
    return response;
};

async function main() {
    // 1. Admin login.
    const loginResponse = await api("/api/auth/v1/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: ADMIN_EMAIL, password: ADMIN_PASSWORD }),
    });
    if (!loginResponse.ok) throw new Error(`admin login failed: ${loginResponse.status}`);
    const { auth_token: token, csrf_token: csrf } = await loginResponse.json();
    const adminHeaders = {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
        "csrf-token": csrf,
    };

    // 2. Create (or reuse) the demo user.
    let userId;
    const createResponse = await api("/api/_admin/user", {
        method: "POST",
        headers: adminHeaders,
        body: JSON.stringify({ email: DEMO_EMAIL, password: DEMO_PASSWORD, verified: true, admin: false }),
    });
    if (createResponse.ok) {
        userId = (await createResponse.json()).id;
        console.log(`[seed] created user ${DEMO_EMAIL} (${userId})`);
    } else if (createResponse.status === 409) {
        const list = await (await api("/api/_admin/user?limit=100", { headers: adminHeaders })).json();
        const found = list.users?.find((u) => u.email === DEMO_EMAIL);
        if (!found) throw new Error("user exists but not found in listing");
        userId = found.id;
        console.log(`[seed] reusing user ${DEMO_EMAIL} (${userId})`);
    } else {
        throw new Error(`create user failed: ${createResponse.status} ${await createResponse.text()}`);
    }

    // 3. Build the seeded knowledge set.
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
        const ulid = `01J${String(index + 1).padStart(4, "0")}AAAAAAAAAAAAAAAAAAAAA`;
        const troubled = index < Math.floor(words.length / 25) + 1;
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

    const knowledgeSetWire =
        "DEFLATE;" + zlib.deflateSync(JSON.stringify(knowledgeSet)).toString("base64");

    console.log("[seed] user ready; domain_user record will be created on first login");
}

main().catch((error) => {
    console.error(error);
    process.exit(1);
});
