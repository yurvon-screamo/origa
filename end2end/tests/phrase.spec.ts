import { test, expect } from "@playwright/test";

interface PhraseData {
    phrases: Phrase[];
}

interface Phrase {
    id: string;
    text: string;
    audio_file: string;
    tokens: unknown[];
    translation_ru: string;
    translation_en: string;
}

test.describe("Phrase Dataset", () => {
    let response_data: PhraseData;

    test.beforeAll(async ({ request }) => {
        const response = await request.get("/phrase/phrase_dataset.json");
        expect(response.ok()).toBeTruthy();
        response_data = await response.json();
    });

    test("phrase dataset loads successfully", async () => {
        expect(response_data.phrases.length).toBe(500);
    });

    test("phrase dataset has required fields", async () => {
        const phrase = response_data.phrases[0];

        expect(phrase).toHaveProperty("id");
        expect(phrase).toHaveProperty("text");
        expect(phrase).toHaveProperty("audio_file");
        expect(phrase).toHaveProperty("tokens");
        expect(phrase).toHaveProperty("translation_ru");
        expect(phrase).toHaveProperty("translation_en");
    });

    test("phrase audio file is accessible", async ({ request }) => {
        const phrase = response_data.phrases[0];
        const audioResponse = await request.get(`/phrase/audio/${phrase.audio_file}`);

        expect(audioResponse.ok()).toBeTruthy();
        expect(audioResponse.headers()["content-type"]).toContain("audio");
    });

    test("phrase audio files are opus format", async () => {
        for (const phrase of response_data.phrases) {
            expect(phrase.audio_file).toMatch(/\.opus$/);
        }
    });

    test("all phrase ids are unique ULIDs", async () => {
        const ids = response_data.phrases.map((p) => p.id);
        expect(new Set(ids).size).toBe(ids.length);
    });

    test("all phrases have non-empty translations", async () => {
        for (const phrase of response_data.phrases) {
            expect(phrase.translation_ru.length).toBeGreaterThan(0);
            expect(phrase.translation_en.length).toBeGreaterThan(0);
        }
    });
});
