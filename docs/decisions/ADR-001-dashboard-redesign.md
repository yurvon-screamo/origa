# ADR-001: Dashboard Redesign — New Home Page Layout

## Status

Accepted

## Date

2026-05-16

## Context

The home page (dashboard) displayed a flat list of stat cards (StatsGrid) with a history modal popup. This did not give users a quick overview of their study progress, daily workload, or recent activity. A redesign was needed to provide:

1. Visual breakdown of JLPT progress by category (Kanji, Words, Grammar)
2. Today's study workload at a glance (new/learning/review counts)
3. 30-day activity trend chart
4. Recently studied cards for quick review

The redesign follows a dark-theme mockup while preserving the existing design system (DESIGN.md): Cormorant Garamond + DM Mono typography, no border-radius, hard offset shadows, earth-tone palette.

## Decision

### Architecture

- **No domain crate changes**: All dashboard data is computed in the UI layer from existing `User` data (`KnowledgeSet.study_cards()`, `lesson_history()`, `jlpt_progress()`)
- **New data computation module** (`dashboard_stats.rs`): `compute_today_overview()`, `compute_recent_studied()`, `compute_30day_chart_data()` — pure functions, no side effects
- **New MultiLineChart component**: SVG chart supporting 3 lines with legend, extends the concept of existing single-line `LineChart`
- **Replaced components**: `StatsGrid` + `QuickStatCard` + `HistoryModal` → `CategoryProgressGrid` + `TodayOverviewCard` + `ActivityChart` + `RecentStudyList`
- **Preserved components**: `WelcomeCard`, `JlptProgressCard`, `content_sync.rs`

### Data Flow

```text
User (from IndexedDB via HybridUserRepository)
    ├── knowledge_set.study_cards() → compute_today_overview(), compute_recent_studied()
    ├── knowledge_set.lesson_history() → compute_30day_chart_data()
    └── jlpt_progress → CategoryProgressGrid (via CategoryProgress)
```

### 30-Day Chart Lines

DailyHistoryItem contains aggregate data (not per-type). Three lines:

1. `known_words` — Learned (green/sage)
2. `in_progress_words` — In Progress (terracotta)
3. `new_words` — New (dark/fg-black)

### Recently Studied

- Sorted by `last_review_date`, limited to 10 items
- Reading field only populated for Kanji cards (via `kun_readings()`)
- Vocabulary/Grammar/Phrase cards show empty reading (dictionary lookup would be too expensive for dashboard)

## Alternatives Considered

### Per-type chart lines (Kanji/words/grammar per day)

- Rejected: `DailyHistoryItem` does not contain per-type breakdown
- Adding per-type tracking would require domain crate changes (out of scope for UI task)

### External chart library (Chart.js, D3)

- Rejected: Project has zero external JS dependencies; self-contained SVG components maintain WASM purity
- Existing `LineChart` proved the SVG approach works

### Keeping StatsGrid alongside new components

- Rejected: Redundant information, cluttered UI
- StatsGrid's detailed stats (positive/negative ratings) can be accessed from profile/settings page in the future

## Consequences

- Dashboard loads all study cards into memory for `compute_today_overview()` — O(n) where n = total cards. Acceptable for <10k cards.
- `MultiLineChart` is a general-purpose component reusable in other contexts (e.g., profile stats page)
- Removed 3 files (~310 lines), added 6 new files (~900 lines), net +590 lines
- E2E tests updated to match new selectors
- All 17 i18n keys added for ru/en locales

## Update v2 (2026-05-16)

### Changes

1. **CategoryProgressGrid moved inside JlptProgressCard accordion**: Instead of a separate block, categories now expand under "Подробнее" toggle inside the JLPT card. Improves information hierarchy and reduces vertical scroll.
2. **Phrase cards excluded from TodayOverview**: `CardType::Phrase` cards are no longer counted in "Обзор на сегодня" statistics, matching the mockup which shows only core study items.
3. **MultiLineChart axis font**: Increased from 9px to 11px for better readability per DESIGN.md.
4. **RecentStudyList layout**: Replaced horizontal scroll with CSS grid (`grid-cols-2/3/5`) for instant visibility of all items without scrolling.
5. **Two-column layout**: `TodayOverviewCard` and `ActivityChart` now sit side-by-side on desktop (`5fr / 7fr`), stacking vertically on mobile.

### Accessibility improvements

- JLPT accordion toggle: `aria-expanded`, `aria-controls`, `role="region"`
- Decorative elements: `aria-hidden="true"` on status dot and chevron
- Removed dead reactive signals reducing memory footprint
