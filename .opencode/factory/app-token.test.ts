// app-token.test.ts — тесты JWT/кэша/конфига токен-рефрешера GitHub App (bun test)
import { test, expect, describe, afterAll } from "bun:test"
import { generateKeyPairSync } from "node:crypto"
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs"
import { join } from "node:path"
import { makeJwt, verifyJwt, loadConfig, readCache, writeCache } from "./app-token"

// /tmp/opencode — препроёвренный каталог среды; голый /tmp изолирован
const tmpDir = (): string => {
  mkdirSync("/tmp/opencode", { recursive: true })
  return mkdtempSync("/tmp/opencode/app-token-test-")
}

const { privateKey, publicKey } = generateKeyPairSync("rsa", { modulusLength: 2048 })
const pem = privateKey.export({ type: "pkcs8", format: "pem" }).toString()

describe("makeJwt", () => {
  test("структура: три base64url-сегмента, RS256, iss=appId", () => {
    const t = makeJwt("12345", pem)
    const parts = t.split(".")
    expect(parts.length).toBe(3)
    const header = JSON.parse(Buffer.from(parts[0], "base64url").toString())
    expect(header.alg).toBe("RS256")
    const payload = JSON.parse(Buffer.from(parts[1], "base64url").toString())
    expect(payload.iss).toBe("12345")
    expect(payload.exp - payload.iat).toBeLessThanOrEqual(600) // лимит GitHub
  })
  test("подпись верифицируется ключом приложения", () => {
    const t = makeJwt("999", pem)
    expect(verifyJwt(t, publicKey)).toBe(true)
  })
  test("чужой ключ подпись не проходит", () => {
    const other = generateKeyPairSync("rsa", { modulusLength: 2048 })
    const t = makeJwt("999", other.privateKey.export({ type: "pkcs8", format: "pem" }).toString())
    expect(verifyJwt(t, publicKey)).toBe(false)
  })
})

describe("loadConfig", () => {
  const dir = tmpDir()
  afterAll(() => rmSync(dir, { recursive: true, force: true }))
  test("валидный конфиг читается", () => {
    const p = join(dir, "ok.json")
    writeFileSync(p, JSON.stringify({ appId: "1", installationId: "2", keyPath: "/x.pem" }))
    expect(loadConfig(p)).toEqual({ appId: "1", installationId: "2", keyPath: "/x.pem" })
  })
  test("битый конфиг (нет поля) — понятная ошибка", () => {
    const p = join(dir, "bad.json")
    writeFileSync(p, JSON.stringify({ appId: "1" }))
    expect(() => loadConfig(p)).toThrow("installationId")
  })
  test("битый JSON — ошибка", () => {
    const p = join(dir, "junk.json")
    writeFileSync(p, "{nope")
    expect(() => loadConfig(p)).toThrow()
  })
  test("пустое поле отклоняется", () => {
    const p = join(dir, "empty.json")
    writeFileSync(p, JSON.stringify({ appId: "", installationId: "2", keyPath: "/x" }))
    expect(() => loadConfig(p)).toThrow("appId")
  })
})

describe("token cache", () => {
  const dir = tmpDir()
  afterAll(() => rmSync(dir, { recursive: true, force: true }))
  const cache = join(dir, "token.json")
  test("свежий кэш отдаётся", () => {
    writeCache(cache, "tok123", new Date(Date.now() + 30 * 60_000).toISOString())
    expect(readCache(cache)).toBe("tok123")
  })
  test("кэш ближе 5 минут к истечению игнорируется", () => {
    writeCache(cache, "tok-old", new Date(Date.now() + 3 * 60_000).toISOString())
    expect(readCache(cache)).toBeNull()
  })
  test("просроченный кэш игнорируется", () => {
    writeCache(cache, "tok-exp", new Date(Date.now() - 1000).toISOString())
    expect(readCache(cache)).toBeNull()
  })
  test("отсутствующий кэш — null, не бросает", () => {
    expect(readCache(join(dir, "nope.json"))).toBeNull()
  })
})
