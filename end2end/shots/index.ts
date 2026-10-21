import type { RegistryShot } from "./registry.ts";

/**
 * Renders the human-readable index of the registry.
 *
 * The index is a view, not a source: the JSON files in `shots/` own the
 * data, and this file is regenerated from them.
 */
export function renderIndex(entries: RegistryShot[]): string {
    const sorted = [...entries].sort((left, right) => {
        const byDate = right.capturedAt.localeCompare(left.capturedAt);
        return byDate !== 0 ? byDate : left.scenario.localeCompare(right.scenario);
    });

    const lines: string[] = [
        "# Свежие скрины приложения",
        "",
        "<!-- Сгенерировано: npm run shots -- index. Не редактировать вручную. -->",
        "",
        "Реестр захватов интерфейса: что снято, когда, из какого коммита и по какой ссылке.",
        "Источник данных — `docs/ui-screenshots/shots/*.json`.",
        "",
        "| Экран | Сценарий | Вьюпорт | Локаль | Размер | Снято | Коммит | Ссылка |",
        "| --- | --- | --- | --- | --- | --- | --- | --- |",
    ];

    for (const entry of sorted) {
        const link = entry.url ? `[png](${entry.url})` : "не загружен";
        lines.push(
            `| ${entry.screen} | ${entry.scenario} | ${entry.viewport} | ${entry.locale} | ${entry.width}×${entry.height} | ${entry.capturedAt} | ${entry.commit.slice(0, 8)} | ${link} |`,
        );
    }

    if (sorted.length === 0) {
        lines.push("| — | — | — | — | — | — | — | — |");
    }

    lines.push("");
    lines.push("## Как обновить");
    lines.push("");
    lines.push("```bash");
    lines.push("cd end2end && npm run shots -- capture --scenario home --label issue-000 --upload");
    lines.push("```");
    return `${lines.join("\n")}\n`;
}
