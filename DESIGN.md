---
version: alpha
name: Origa Design System
colors:
  primary: "#3d4535"
  secondary: "#a65d3f"
  tertiary: "#8b7355"
  neutral: "#6b665a"
  surface: "#f7f4ee"
  on-surface: "#1a1915"
  error: "#8b4545"
  success: "#4a6b4a"
  warning: "#a67c3f"
  bg-cream: "#f7f4ee"
  bg-paper: "#fdfbf7"
  bg-aged: "#ede8dd"
  bg-warm: "#f0ebe1"
  fg-black: "#1a1915"
  fg-muted: "#6b665a"
  fg-light: "#9a9590"
  border-light: "#d4cfc3"
  border-dark: "#1a1915"
  accent-olive: "#3d4535"
  accent-terracotta: "#a65d3f"
  accent-gold: "#8b7355"
  accent-sage: "#6b7b5e"
typography:
  headline-lg:
    fontFamily: "Cormorant Garamond"
    fontSize: "48px"
    fontWeight: 300
    lineHeight: 1.1
    letterSpacing: "-0.01em"
  headline-md:
    fontFamily: "Cormorant Garamond"
    fontSize: "32px"
    fontWeight: 400
    lineHeight: 1.2
  headline-sm:
    fontFamily: "Cormorant Garamond"
    fontSize: "24px"
    fontWeight: 500
    lineHeight: 1.2
  body-lg:
    fontFamily: "DM Mono"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "0.02em"
  body-md:
    fontFamily: "DM Mono"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "0.05em"
  label-md:
    fontFamily: "DM Mono"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.1em"
    textTransform: uppercase
  label-sm:
    fontFamily: "DM Mono"
    fontSize: "10px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.15em"
    textTransform: uppercase
rounded:
  none: 0px
  sm: 2px
  md: 4px
  lg: 8px
spacing:
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
  xxl: 48px
  xxxl: 64px
border:
  width-default: 1px
  width-thick: 2px
  color-default: "{colors.border-dark}"
  color-light: "{colors.border-light}"
shadow:
  offset-x: 6px
  offset-y: 6px
  color: "{colors.border-dark}"
components:
  button-primary:
    backgroundColor: "{colors.fg-black}"
    textColor: "{colors.bg-paper}"
    borderWidth: "1px"
    borderColor: "{colors.fg-black}"
    borderRadius: "{rounded.none}"
    padding: "8px 12px"
    fontFamily: "{typography.label-md.fontFamily}"
    fontSize: "10px"
    textTransform: uppercase
    letterSpacing: "0.1em"
  button-secondary:
    backgroundColor: "{colors.bg-paper}"
    textColor: "{colors.fg-black}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    borderRadius: "{rounded.none}"
    padding: "8px 12px"
    fontFamily: "{typography.label-md.fontFamily}"
    fontSize: "10px"
  button-olive:
    backgroundColor: "{colors.accent-olive}"
    textColor: "{colors.bg-paper}"
    borderWidth: "1px"
    borderColor: "{colors.accent-olive}"
  input-field:
    backgroundColor: "{colors.bg-paper}"
    textColor: "{colors.fg-black}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    borderRadius: "{rounded.none}"
    padding: "12px 16px"
    fontFamily: "{typography.body-md.fontFamily}"
    fontSize: "12px"
    focusShadow: "4px 4px 0 {colors.border-dark}"
  card:
    backgroundColor: "{colors.bg-paper}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    padding: "24px"
    shadowOffsetX: "6px"
    shadowOffsetY: "6px"
    shadowBorderWidth: "1px"
    shadowBackground: "{colors.bg-aged}"
  tag:
    backgroundColor: "{colors.bg-paper}"
    textColor: "{colors.fg-black}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    padding: "6px 12px"
    fontFamily: "{typography.label-md.fontFamily}"
    fontSize: "10px"
    textTransform: uppercase
    letterSpacing: "0.2em"
  tag-olive:
    backgroundColor: "{colors.accent-olive}"
    textColor: "{colors.bg-paper}"
    borderColor: "{colors.accent-olive}"
  tag-terracotta:
    backgroundColor: "{colors.accent-terracotta}"
    textColor: "{colors.bg-paper}"
    borderColor: "{colors.accent-terracotta}"
  modal-backdrop:
    backgroundColor: "rgba(26, 25, 21, 0.6)"
  modal-content:
    backgroundColor: "{colors.bg-paper}"
    borderWidth: "1px"
    borderColor: "{colors.border-dark}"
    borderRadius: "{rounded.none}"
    padding: "32px"
    maxWidth: "640px"
    shadowOffset: "8px"
    shadowBackground: "{colors.bg-aged}"
animation:
  duration-fast: "100ms"
  duration-normal: "200ms"
  duration-slow: "300ms"
  duration-entrance: "350ms"
  ease-paper: "cubic-bezier(0.25, 0.46, 0.45, 0.94)"
  ease-out-expo: "cubic-bezier(0.16, 1, 0.3, 1)"
  ease-out-back: "cubic-bezier(0.34, 1.56, 0.64, 1)"
---

# DESIGN.md — Origa Design System

## Overview

Origa — минималистичный интерфейс для приложения изучения
японского языка, вдохновлённый эстетикой японской типографики
и традиционной бумаги. Дизайн отличается низким визуальным
шумом, чёткой геометрией, тёплой землистой палитрой и
авторитетным моноширинным типографическим голосом.

**Концепция**: "Типографика печатного станка на японской бумаге".
Все интерактивные элементы используют 1px чёрные границы без
скруглений. Тени реализованы через псевдо-элементы `::after`
со смещением и линией границы — эффект "сдвига" вместо
размытия. Типографика — это контраст между элегантным serif
(Cormorant Garamond) для заголовков/японского текста и
утилитарным monospace (DM Mono) для всего интерфейса.

## Colors

Палитра построена на тёплых кремовых фонах
(`bg-cream`, `bg-paper`, `bg-aged`) с почти чёрным текстом
(`fg-black`). Акцентные цвета — приглушённые оливковый,
терракотовый и золотой, которые создают ощущение
традиционных японских красок без яркости.

- **Primary** (`#3d4535`): CTA-кнопки, призыв к действию,
  статус "в процессе", активные состояния. Используется
  умеренно — только когда нужен цветовой акцент.
- **Secondary** (`#a65d3f`): Предупреждения,
  "штамп"-эффекты, hover-ссылки. Визуально заметен,
  привлекает внимание к важным меткам.
- **Tertiary** (`#8b7355`): Менее важные акценты, вторичные метаданные.
- **Surface** (`#f7f4ee`): Основной фон приложения.
  Тёплый, не чисто белый.
- **On-surface** (`#1a1915`): Основной читаемый текст.
  Почти чёрный, но с тёплым подтоном.

### Цветовые правила

1. **Границы всегда чёрные**: Даже при использовании
   цветных вариантов кнопок (olive, terracotta),
   hover-эффекты возвращаются к чёрно-белому контрасту.
2. **Не используйте `border-radius`**: Компоненты имеют
   углы 0px. Единственное исключение — radio (50%) и
   небольшой `border-radius: 2px` для utility-элементов.
3. **Не используйте `box-shadow` с размытием**: Все тени —
   "hard shadows" через offset pseudo-elements.

## Typography

### Система шрифтов

- **Display**: Cormorant Garamond, 300,
  clamp(1.75rem, 6vw, 3rem) — H1, hero titles
- **Heading**: Cormorant Garamond, 400-500,
  1.25–1.5rem — H2–H6, Japanese text display
- **Body**: DM Mono, 400, 12px —
  Основной текст, описания
- **Label**: DM Mono, 400, 9–11px —
  Кнопки, теги, навигация
- **Mono**: DM Mono, 400-500, 11px —
  Код, технические данные

### Правила типографики

- **Заголовки** (Cormorant Garamond) используют
  `letter-spacing: -0.01em` для естественной плотности.
- **Интерфейсные элементы** (DM Mono) используют
  `text-transform: uppercase` + `letter-spacing: 0.1em` —
  это создаёт эффект печатного набора.
- **Размеры шрифта для UI-компонентов** никогда не превышают
  12px — приложение использует очень мелкий, плотный
  типографический масштаб.

## Layout & Spacing

### Принципы компоновки

- **No border-radius**: Все контейнеры — строго прямоугольные.
- **Hard shadows**: Карточки и модальные окна имеют "тень"
  как отдельный offset-слой с границей, а не CSS box-shadow.
- **Отступы**: Используется 8px сетка (4, 8, 16, 24, 32, 48, 64, 96px).
- **Контейнеры**:
  - `card-layout-small`: max-width 480px
  - `card-layout-medium`: max-width 768px
  - `card-layout-large`: max-width 1024px
  - `card-layout-adaptive`: width 100%

### Responsive Breakpoints

| Breakpoint | Ширина     | Ключевые изменения                          |
| ---------- | ---------- | ------------------------------------------- |
| Mobile     | < 640px    | Single column, bottom nav, compact padding  |
| Tablet     | 640–1023px | Multi-column grids, medium padding (24px)   |
| Desktop    | ≥ 1024px   | Full layouts, large padding (32px)          |

**Safe Areas**: Все fixed-элементы учитывают
`env(safe-area-inset-*)` для iPhone notch.

## Elevation & Depth

Все эффекты глубины достигаются через
**псевдо-элементы** с чёткими границами, а не blur-модели.

```text
Card Shadow:
┌──────────────┐
│   Content    │
│              │
└──────────────┘
         ┌──────────────┐
         │  1px border  │  ← Offset 6px right, 6px bottom
         │  bg-aged     │
         └──────────────┘
```

- **Card shadow**:
  `bottom: -6px; right: -6px; border: 1px solid
  var(--border-dark); background: var(--bg-aged)`
- **Modal shadow**: `bottom: -8px; right: -8px;`
- **Toast shadow**:
  `box-shadow: 4px 4px 0 var(--border-dark)`
  (единственное место с box-shadow — toast)

## Shapes

- **Buttons**: Прямоугольники,
  padding `8px 12px` (mobile) / `14px 28px` (desktop).
- **Inputs**: Прямоугольники, focus-эффект —
  `box-shadow: 4px 4px 0 var(--border-dark)`.
- **Tags**: Inline-flex, прямоугольные,
  плотный letter-spacing.
- **Checkbox/Radio**: Square (20×20) / Circle (20×20),
  checked через inset square/circle.
- **Toggle**: Square track (44×24),
  square thumb with border.

**Запрещено**: `border-radius` на основных компонентах
(карточки, кнопки, модальные окна, формы).

## Components

### Button

```css
.btn {
  font-family: "DM Mono", monospace;
  font-size: 9px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  padding: 8px 12px;
  border: 1px solid var(--border-dark);
  background: var(--bg-paper);
  color: var(--fg-black);
  cursor: pointer;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
```

Варианты:

- **Default**: Белый фон, чёрная граница.
  Hover: инверсия (чёрный фон, белый текст).
- **Filled**: Чёрный фон, белый текст. Hover: инверсия.
- **Olive**: Оливковый фон/граница. Hover: белый фон, оливковый текст.
- **Ghost**: Прозрачный фон, без границы на покое. Hover: белый фон + граница.

Hover-эффект: псевдо-элемент `::before` с
`inset: 2px` и `border: 1px` появляется с opacity 0→1.

### Card

```css
.card {
  background: var(--bg-paper);
  border: 1px solid var(--border-dark);
  padding: 24px;
  position: relative;
}
```

- `.card-shadow` — добавляет offset shadow через `::after`.
- `.card-selectable` — hover меняет фон на `bg-aged`.
- `.selected` — граница `accent-olive`, фон `bg-warm`.

### Modal

```css
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(26, 25, 21, 0.6);
  z-index: 100;
}

.modal-content {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--bg-paper);
  border: 1px solid var(--border-dark);
  padding: 32px;
  max-width: 640px;
  width: 90%;
  z-index: 101;
}
```

Модальное окно имеет offset shadow (`::after`) и
поддерживает анимации входа/выхода
(`anima-modal-enter` / `anima-modal-exit`).
Закрывается по Escape и click на backdrop.

### Drawer

- **Mobile** (<640px): выезжает снизу, height 80vh, border-top.
- **Desktop** (≥640px): выезжает справа, width 80vw max 800px, border-right.
- Оба варианта имеют offset shadow и анимацию slide.

### Input

```css
.input-field {
  font-family: "DM Mono", monospace;
  font-size: 12px;
  padding: 12px 16px;
  border: 1px solid var(--border-dark);
  background: var(--bg-paper);
  color: var(--fg-black);
  width: 100%;
}

.input-field:focus {
  outline: none;
  box-shadow: 4px 4px 0 var(--border-dark);
}

.input-field::placeholder {
  color: var(--fg-muted);
  text-transform: uppercase;
  font-size: 10px;
  letter-spacing: 0.1em;
}
```

Placeholder всегда uppercase с letter-spacing — это фирменный паттерн.

### Tag

Категориальная метка. Бывает интерактивной (фильтр/переключатель)
и неинтерактивной (статус/категория).

**Non-interactive Tag** (без `on_click`):

```css
.tag {
  font-family: "DM Mono", monospace;
  font-size: 9px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  padding: 6px 12px;
  border: 1px solid var(--border-light);
  background: var(--bg-paper);
  color: var(--fg-muted);
}
```

Светлая рамка и тусклый текст — визуально "лёгкий" элемент.
Не имеет hover-эффектов. Рендерится как `<span>`.

**Clickable Tag** (с `on_click`, класс `tag-clickable`):

```css
.tag-clickable {
  border-color: var(--border-dark);
  color: var(--fg-black);
  cursor: pointer;
}
.tag-clickable:hover {
  background: var(--fg-black);
  color: var(--bg-paper);
  transform: translateY(-1px);
}
.tag-clickable:active {
  transform: translateY(1px) scale(0.98);
}
```

Чёрная рамка, color flip hover, press-эффект на active.
Рендерится как `<button>`. Имеет `::before` inner border на hover
и `anima-focus-ring` для keyboard navigation.

Варианты: `tag-filled`, `tag-olive`, `tag-terracotta`, `tag-sage`.
Все варианты инвертируются на hover (color flip).

### Badge

Неинтерактивный статус-индикатор. Никогда не кликабельный.

```css
.badge {
  font-family: "DM Mono", monospace;
  font-size: 10px;
  letter-spacing: 0.05em;
  padding: 2px 8px;
  border: none;
  background: var(--bg-aged);
  color: var(--fg-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
}
```

Badge — это "штамп на полях". Без рамки, приглушённые цвета.
Никогда не имеет `cursor: pointer` или hover-эффектов.

### Interaction Hierarchy

Визуальная шкала интерактивности (Paper Gradient).
Чем "тяжелее" элемент визуально, тем он интерактивнее.

<!-- markdownlint-disable MD013 -->
| Уровень | Компонент            | Рамка          | Фон       | Текст      | Интерактивный |
|---------|----------------------|----------------|-----------|------------|---------------|
| 0       | Inline               | None           | None      | `fg-muted` | Нет           |
| 1       | Badge                | None           | `bg-aged` | `fg-muted` | Нет           |
| 2       | Tag                  | `border-light` | `bg-paper`| `fg-muted` | Нет           |
| 3       | Chip (tag-clickable) | `border-dark`  | `bg-paper`| `fg-black` | Да            |
| 4       | Button               | `border-dark`  | `bg-paper`| `fg-black` | Да            |
<!-- markdownlint-enable MD013 -->

Принцип: чёрная рамка (`border-dark`) = можно нажать.
Светлая рамка или отсутствие рамки = нельзя нажать.

### Furigana

```css
.furigana-ruby {
  font-family: "Cormorant Garamond", serif;
  ruby-align: center;
}

.furigana-rt {
  font-family: "DM Mono", monospace;
  font-size: 0.5em;
  color: var(--fg-muted);
}
```

Основной японский текст — Cormorant Garamond
(serif создаёт классический вид иероглифов).
Furigana (ромуадзи/кана) — DM Mono в 50% размера.

## Animation System

### Naming Convention

Все анимации используют префикс `anima-*`. Это позволяет
быстро находить анимации в коде и применять их через CSS-классы.

### Категории

- **Standard**: `anima-press` (кнопки), `anima-lift` (карточки),
  `anima-reveal` (навигация) — унифицированные hover/active эффекты
- **Hover**: `anima-card-lift` (deprecated→anima-lift),
  `anima-tag-hover` (deprecated→tag-clickable),
  `anima-link-typewriter`, `anima-btn-press` (deprecated→anima-press),
  `anima-avatar-hover` — Микро-интеракции при наведении
- **Entrance**: `anima-modal-enter`,
  `anima-backdrop-enter`, `anima-toast-bounce`,
  `anima-slide-up` — Появление элементов
- **Exit**: `anima-modal-exit`, `anima-backdrop-exit`,
  `anima-toast-exit` — Исчезновение элементов
- **Micro**: `anima-shake`, `anima-check-draw`,
  `anima-strikethrough` — Отклик на действия
- **Loading**: `anima-skeleton-paper`,
  `anima-spinner-square` — Состояния загрузки
- **Japanese**: `anima-kanji-reveal`,
  `anima-furigana-show` — Для японского языка
- **Layout**: `anima-stagger`, `anima-page-fade`,
  `anima-slide-up` — Каскадное появление
- **Special**: `anima-stamp`, `anima-pulse-subtle`,
  `anima-focus-ring` — Эффекты внимания и фокуса

### Easing Functions

- `anima-ease-paper`:
  `cubic-bezier(0.25, 0.46, 0.45, 0.94)` —
  Стандартные transitions
- `anima-ease-out-expo`:
  `cubic-bezier(0.16, 1, 0.3, 1)` —
  Entrance анимации
- `anima-ease-out-back`:
  `cubic-bezier(0.34, 1.56, 0.64, 1)` —
  Bounce эффекты (toast)

### Standard Animation Classes

Три уровня анимаций для разных семантических ролей:

- **anima-press** — кнопки и действия.
  Физическая метафора: клавиша пишущей машинки.
  Active: `translateY(1px) scale(0.98)`.
  Применяется к: Button, IconBtn, Pagination.

- **anima-lift** — карточки и объекты.
  Физическая метафора: лист бумаги приподнимается.
  Hover: `translateY(-2px)`. Active: `translateY(0)`.
  Применяется к: Card (interactive).

- **anima-reveal** — навигация и ссылки.
  Физическая метафора: маркер выделяет пункт.
  Только opacity/color/bg transition, без transform.
  Применяется к: Sidebar, BottomTab, Breadcrumbs.

### Accessibility

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

## Do's and Don'ts

### ✅ Do

- Используйте `1px solid var(--border-dark)` для границ всех компонентов.
- Применяйте `text-transform: uppercase` + `letter-spacing`
  к всему интерфейсному тексту (кнопки, теги, навигация).
- Используйте `Cormorant Garamond` только для заголовков и японского текста.
- Добавляйте offset shadow через `::after` для карточек и модальных окон.
- Используйте `bg-paper` для контента поверх `bg-cream`.
- Применяйте `anima-focus-ring` для доступной навигации с клавиатуры.

### ❌ Don't

- **Не используйте `border-radius`** на основных компонентах
  (кнопки, карточки, модалки, формы). Единственные
  исключения: checkbox/radio активные индикаторы и
  utility-классы.
- **Не используйте `box-shadow` с blur** (например,
  `box-shadow: 0 4px 6px rgba(...)`). Все тени — hard offset.
- **Не используйте яркие цвета** (pure red/green/blue).
  Акцентные цвета Origa — всегда muted/приглушённые.
- **Не используйте шрифты sans-serif** (Inter, Roboto) —
  только Cormorant Garamond и DM Mono.
- **Не создавайте полупрозрачные фоны** для overlay, кроме
  `modal-backdrop` и `loading-overlay`.
- **Не задавайте font-size > 12px** для интерфейсных элементов
  (кнопки, навигация, теги). Body-текст: 12px, labels: 9–11px.

## Paper Texture

Фоновая текстура "бумаги" реализована через fixed
pseudo-element с SVG noise:

```css
.paper-texture::before {
  content: "";
  position: fixed;
  inset: 0;
  background-image: url("data:image/svg+xml,...feTurbulence...");
  opacity: 0.03;
  pointer-events: none;
  z-index: 0;
}
```

Это создаёт едва заметную зернистость, придающую теплоту
интерфейсу. Элемент применяется к `<body>`.

## Special Patterns

### Stamp Effect

```css
.stamp {
  font-family: "Cormorant Garamond", serif;
  font-size: 11px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  padding: 8px 16px;
  border: 2px solid var(--accent-terracotta);
  color: var(--accent-terracotta);
  transform: rotate(-3deg);
}
```

Используется для важных меток/статусов. Имеет анимацию
"штамп" (`anima-stamp`) с bounce.

### Kanji Drawing Animation

```css
.kanji-animation-svg path:not(.bg) {
  stroke-dasharray: 1000;
  stroke-dashoffset: 1000;
  animation: kanji-draw 0.4s linear forwards;
}
```

SVG анимация порядка начертания иероглифа — последовательное "рисование" черт.

### Quiz Options

```css
.quiz-option-neutral   /* default */
.quiz-option-correct   /* success color */
.quiz-option-wrong     /* error color */
.quiz-option-dimmed    /* opacity 0.5 */
```

Три состояния для викторин: нейтральное, правильный ответ
(зелёная граница + bg-warm), неправильный ответ
(красная граница + bg-warm). Неправильный ответ
подсвечивается, остальные диммируются.
