#!/usr/bin/env node
/**
 * Builds .stitch/storage-state.json for `stitch capture --storage`:
 * signs in the demo user via API and writes the app's session storage
 * (trailbase_session in gloo JSON form) + RU locale cookie.
 */
import { readFileSync, writeFileSync } from "node:fs";

const env = Object.fromEntries(
    readFileSync(new URL("../.env", import.meta.url), "utf8")
        .split("\n")
        .filter((l) => l.includes("="))
        .map((l) => [l.slice(0, l.indexOf("=")).trim(), l.slice(l.indexOf("=") + 1).trim()]),
);
const BASE = process.env.TRAILBASE_URL || env.TRAILBASE_URL;

const login = await fetch(`${BASE}/api/auth/v1/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
        email: process.env.DEMO_EMAIL || "insights2@origa.local",
        password: "e2e-test-password-123",
    }),
});
if (!login.ok) throw new Error(`login failed: ${login.status}`);
const data = await login.json();

// trailbase_id: base64 of the JWT sub (UUID bytes).
const claims = JSON.parse(
    Buffer.from(data.auth_token.split(".")[1], "base64").toString("utf8"),
);
const trailbaseId = Buffer.from(claims.sub, "base64").toString("hex")
    .replace(/^(.{8})(.{4})(.{4})(.{4})(.{12})$/, "$1-$2-$3-$4-$5");

const session = {
    auth_token: data.auth_token,
    refresh_token: data.refresh_token,
    email: process.env.DEMO_EMAIL || "insights2@origa.local",
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
console.log("storage state written; trailbase_id:", trailbaseId, "| exp:", claims.exp);
