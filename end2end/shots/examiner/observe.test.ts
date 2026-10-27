import assert from "node:assert/strict";
import { test } from "node:test";

import { formatDomSnapshot } from "./observe.ts";

const NODES = [
    { tag: "div", testid: "home-page", role: null, text: "Главная" },
    { tag: "h1", testid: null, role: null, text: "Добро пожаловать" },
    { tag: "button", testid: "start-lesson", role: null, text: "Начать урок" },
    { tag: "p", testid: null, role: null, text: "確認します — подтверждаю" },
];

test("the snapshot leads with the route/locale/viewport header", () => {
    const snapshot = formatDomSnapshot({ route: "/home", locale: "ru-RU", viewport: "desktop", nodes: NODES });
    assert.ok(snapshot.text.startsWith("route=/home locale=ru-RU viewport=desktop\n"));
    assert.equal(snapshot.truncatedNodes, 0);
    assert.equal(snapshot.nodeCount, 4);
});

test("marked nodes carry their testid, unmarked ones do not", () => {
    const snapshot = formatDomSnapshot({ route: "/home", locale: "ru-RU", viewport: "desktop", nodes: NODES });
    const lines = snapshot.text.split("\n").slice(1);
    assert.equal(lines[0], "div #home-page — Главная");
    assert.equal(lines[1], "h1 — Добро пожаловать");
    assert.equal(lines[3], "p — 確認します — подтверждаю");
});

test("the snapshot stops at the character budget and says how much was cut", () => {
    const many = Array.from({ length: 100 }, (_, index) => ({
        tag: "div",
        testid: `node-${index}`,
        role: null,
        text: `Длинный текст элемента номер ${index} с подробностями`,
    }));
    const snapshot = formatDomSnapshot(
        { route: "/home", locale: "ru-RU", viewport: "desktop", nodes: many },
        400,
    );
    assert.ok(snapshot.text.length <= 400 + 40);
    assert.ok(snapshot.text.includes("more elements truncated"));
    assert.ok(snapshot.truncatedNodes > 0);
    assert.equal(snapshot.nodeCount + snapshot.truncatedNodes, 100);
});

test("a role attribute is part of the line", () => {
    const snapshot = formatDomSnapshot({
        route: "/lesson",
        locale: "ru-RU",
        viewport: "mobile",
        nodes: [{ tag: "div", testid: "dialog", role: "dialog", text: "" }],
    });
    assert.ok(snapshot.text.includes("div #dialog role=dialog"));
});
