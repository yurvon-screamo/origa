// app-token.ts — installation access token GitHub App для uwuwu-agent[bot].
// Вызывается обёрткой gh-as-agent (и credential helper'ом ворктри):
// печатает валидный токен в stdout. Токены живут 1 час — кэшируем в /tmp
// до истечения минус 5 минут. JWT подписывается приватным ключом приложения.
//
// Конфиг: ~/.config/origa-factory/app.json
//   { "appId": "123456", "installationId": "78901234",
//     "keyPath": "/home/yurvon/.config/origa-factory/uwuwu-agent.pem" }

import { createSign, createVerify, generateKeyPairSync, type KeyObject } from "node:crypto"
import { readFileSync, writeFileSync, mkdirSync } from "node:fs"
import { dirname, join } from "node:path"
import { homedir } from "node:os"

export interface AppConfig {
  appId: string
  installationId: string
  keyPath: string
}

export const b64url = (data: string | Buffer): string => Buffer.from(data).toString("base64url")

/** RS256-JWT приложения (iss=appId, окно жизни 3 мин — GitHub требует 10 мин максимум). */
export function makeJwt(appId: string, keyPem: string, nowSec: number = Math.floor(Date.now() / 1000)): string {
  const header = b64url(JSON.stringify({ alg: "RS256", typ: "JWT" }))
  const payload = b64url(JSON.stringify({ iat: nowSec - 60, exp: nowSec + 180, iss: appId }))
  const signature = b64url(createSign("RSA-SHA256").update(`${header}.${payload}`).sign(keyPem))
  return `${header}.${payload}.${signature}`
}

export function verifyJwt(token: string, publicKey: KeyObject): boolean {
  const [h, p, s] = token.split(".")
  if (!h || !p || !s) return false
  return createVerify("RSA-SHA256").update(`${h}.${p}`).verify(publicKey, Buffer.from(s, "base64url"))
}

export function loadConfig(path: string): AppConfig {
  const raw = JSON.parse(readFileSync(path, "utf-8")) as Partial<AppConfig>
  for (const k of ["appId", "installationId", "keyPath"] as const) {
    if (typeof raw[k] !== "string" || (raw[k] as string) === "") {
      throw new Error(`app.json: поле ${k} обязательно`)
    }
  }
  return raw as AppConfig
}

interface CachedToken { token: string; expiresAtMs: number }

export function readCache(path: string, nowMs: number = Date.now()): string | null {
  try {
    const c = JSON.parse(readFileSync(path, "utf-8")) as CachedToken
    // запас 5 минут: не подставить агенту токен, истекающий посреди прогона
    if (typeof c.token === "string" && c.token !== "" && nowMs < c.expiresAtMs - 5 * 60_000) return c.token
  } catch { /* нет/битый кэш — обновим */ }
  return null
}

export function writeCache(path: string, token: string, expiresAtIso: string): void {
  mkdirSync(dirname(path), { recursive: true })
  writeFileSync(path, JSON.stringify({ token, expiresAtMs: Date.parse(expiresAtIso) } satisfies CachedToken))
}

/** Запрос свежего installation token у GitHub. */
export async function fetchInstallationToken(
  cfg: AppConfig,
  jwt: string,
): Promise<{ token: string; expiresAt: string }> {
  const res = await fetch(`https://api.github.com/app/installations/${cfg.installationId}/access_tokens`, {
    method: "POST",
    headers: { authorization: `Bearer ${jwt}`, accept: "application/vnd.github+json" },
  })
  if (!res.ok) {
    throw new Error(`GitHub API ${res.status}: ${(await res.text()).slice(0, 200)}`)
  }
  const j = (await res.json()) as { token?: unknown; expires_at?: unknown }
  if (typeof j.token !== "string" || typeof j.expires_at !== "string") {
    throw new Error("GitHub API: неожиданный формат ответа")
  }
  return { token: j.token, expiresAt: j.expires_at }
}

const CFG_PATH = join(homedir(), ".config/origa-factory/app.json")
const CACHE_PATH = "/tmp/opencode/origa-factory/app-token.json"

const main = async (): Promise<void> => {
  const cfg = loadConfig(CFG_PATH)
  const cached = readCache(CACHE_PATH)
  if (cached !== null) {
    console.log(cached)
    return
  }
  const keyPem = readFileSync(cfg.keyPath, "utf-8")
  const fresh = await fetchInstallationToken(cfg, makeJwt(cfg.appId, keyPem))
  writeCache(CACHE_PATH, fresh.token, fresh.expiresAt)
  console.log(fresh.token)
}

// при импорте в тесты main не запускаем
if (import.meta.main) {
  main().catch((e: unknown) => {
    console.error(`app-token: ${e instanceof Error ? e.message : e}`)
    process.exit(1)
  })
}
