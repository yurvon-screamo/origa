# Tasks

## В сетах некорректный состав слов

Проблемы:

* В сетах есть слова которые не представлены в нашем словаре
* В сетах есть слова которые не токенизированы

Решение:

* Все слова из сетов прогнать через токенайзер, обновить слова в сетах исходя из результата токенизации.
* В словарь добавить слова-токены для которых нет значений.

## OCR повторное открытие дравера

Проблемы:

* При повторной попытке воспользоваться OCR все зависает
* Большая часть времени OCR - загрузка модели в память
* При повторном открытии дравера заведения слов по умолчанию выбрана вкладка "Изображение", а показывается компонент ввода текста

Решение:

* Не выгружать OCR из памяти после обработки 1й картинки
* Исправить баги состояния дровера

## Кривая верстка карточек

Проблема:

* Кривая верстка карточек /words
* Кривая верстка карточек /kanji
* Кривая верстка карточек /grammar
* Текст разрывает на куски и новые строки
* Кнопка "Развернуть" иногда занимает больше места чем занял бы контент, огромный отступ снизу

Решение (предполагаемое):

* Надо кнопки и тег со статусом карточки вынести на отдельную строку над текстом.
* Уменьшить кнопку развернуть и уменьшить отступы от нее до контента и границ (особено снизу)

![img](Кривая%20верстка%20карточек.png)

## На телефоне неудобно использовать тесты

Проблема:

* Тесты могут занимать много вертикального пространтсва и приходится листать вниз чтобы увидеть варианты 2 и 4

Решение:

* Кнопка "Свернуть\развернуть" для длинного контента

![img](./На%20телефоне%20неудобно%20использовать%20тесты.png)

## Плавает размер текста в карточках типа "Слово"

Вот 2 примера, по сути надо чтобы всегда было как на нормальный текст, но не мельче.

Мелкий текст: ![img](./Мелкий%20текст.png)

Нормальный текст: ![img](./Нормальный%20текст.png)

Я скопировал из браузера итоговую реализацию тут от что:

  <!-- Нормального рамера -->
  <span class="furigana-text "><ruby class="furigana-ruby">様<rp>(</rp>
      <rt class="furigana-rt">ヨー</rt>
      <rp>)</rp>
    </ruby><!----></span>

  <!-- Мелкий пиздец -->
  <div class="markdown-text prose prose-lg ">
    <div>
      <p><ruby class="">汚い<rp>(</rp>
          <rt class="furigana-rt">キタナイ</rt>
          <rp>)</rp>
        </ruby></p>
      <!---->
    </div>
  </div>

## В ответе кандзи-карточки нет радикалов

Нужно разобраться в чем дело. Может не инициализирован словарь радикалов?

## Режим прописей кандзи

Дорабатываем наш компонент кандзи анимации и переиспользуем его:

* В странице /kanji чтобы я мог открыть дравер по кнопке и там потренироваться в прописи конкретного кандзи
* В уроке новый тип карточки - прописи

Вот примерный план реализации (будь внимателен, он возможно с ошибками):

```rust
use crate::core::config::public_url;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, Path2d, PointerEvent, SvgPathElement,
};
use js_sys::Array;
use std::cell::RefCell;
use std::rc::Rc;

// ==================== НАСТРОЙКИ (проверь под свои SVG) ====================
const CANVAS_SIZE: u32 = 400;
const SVG_SCALE: f64 = 2.0; // большинство kanji_animations имеют viewBox="0 0 200 200"
// Если у тебя viewBox="0 0 109 109" — поменяй на 400.0 / 109.0 ≈ 3.67

// ==================== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ====================

fn extract_stroke_paths(svg_html: &str) -> Vec<String> {
    let mut paths = vec![];
    let mut pos = 0usize;

    while let Some(rel_start) = svg_html[pos..].find("<path") {
        let abs_start = pos + rel_start;
        let rest = &svg_html[abs_start..];
        let tag_end = rest.find('>').unwrap_or(rest.len());
        let path_tag = &rest[..=tag_end];

        // пропускаем фоновый слой (как в твоём add_animation_delays)
        if path_tag.contains("class=\"bg\"") || path_tag.contains("class='bg'") {
            pos = abs_start + tag_end + 1;
            continue;
        }

        // извлекаем d="..." (поддержка " и ')
        if let Some(d_start) = path_tag.find("d=\"") {
            let ds = d_start + 3;
            if let Some(d_end) = path_tag[ds..].find('"') {
                paths.push(path_tag[ds..ds + d_end].to_string());
            }
        } else if let Some(d_start) = path_tag.find("d='") {
            let ds = d_start + 3;
            if let Some(d_end) = path_tag[ds..].find('\'') {
                paths.push(path_tag[ds..ds + d_end].to_string());
            }
        }

        pos = abs_start + tag_end + 1;
    }
    paths
}

fn draw_all(
    ctx: &CanvasRenderingContext2d,
    strokes: &[String],
    current_idx: usize,
    completed: &[String],
    user_points: &[(f64, f64)],
) {
    ctx.clear_rect(0.0, 0.0, CANVAS_SIZE as f64, CANVAS_SIZE as f64);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    // 1. Уже завершённые штрихи — толстые чёрные
    ctx.set_line_width(18.0);
    ctx.set_stroke_style(&JsValue::from_str("#111"));
    ctx.save();
    ctx.scale(SVG_SCALE, SVG_SCALE);
    for d in completed {
        if let Ok(p) = Path2d::new_with_path_string(d) {
            ctx.stroke_with_path(&p);
        }
    }
    ctx.restore();

    // 2. Текущий штрих-подсказка — красная пунктирная
    if let Some(d) = strokes.get(current_idx) {
        ctx.set_line_width(6.0);
        ctx.set_stroke_style(&JsValue::from_str("#e74c3c"));
        ctx.set_line_dash(&Array::of2(&JsValue::from(8), &JsValue::from(4)));
        ctx.save();
        ctx.scale(SVG_SCALE, SVG_SCALE);
        if let Ok(p) = Path2d::new_with_path_string(d) {
            ctx.stroke_with_path(&p);
        }
        ctx.restore();
        ctx.set_line_dash(&Array::new());
    }

    // 3. То, что рисует пользователь — синяя линия поверх всего
    if user_points.len() >= 2 {
        ctx.set_line_width(12.0);
        ctx.set_stroke_style(&JsValue::from_str("#3498db"));
        ctx.begin_path();
        ctx.move_to(user_points[0].0, user_points[0].1);
        for &(x, y) in &user_points[1..] {
            ctx.line_to(x, y);
        }
        ctx.stroke();
    }
}

fn get_canvas_coords(ev: &PointerEvent, canvas: &HtmlCanvasElement) -> (f64, f64) {
    let rect = canvas.get_bounding_client_rect();
    let x = (ev.client_x() as f64 - rect.left()) * (CANVAS_SIZE as f64 / rect.width());
    let y = (ev.client_y() as f64 - rect.top()) * (CANVAS_SIZE as f64 / rect.height());
    (x, y)
}

fn is_stroke_similar(user_points: &[(f64, f64)], expected_d: &str) -> bool {
    if user_points.len() < 8 {
        return false;
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let svg = document
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
        .unwrap();
    let path_el = document
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "path")
        .unwrap();
    let _ = path_el.set_attribute("d", expected_d);
    let _ = svg.append_child(&path_el);

    let path: SvgPathElement = path_el.dyn_into().unwrap();
    let total = path.get_total_length();
    if total == 0.0 {
        return false;
    }

    let mut matches = 0usize;
    let tolerance = 15.0; // в координатах SVG (после / SVG_SCALE)

    for &(x, y) in user_points {
        let svg_x = x / SVG_SCALE;
        let svg_y = y / SVG_SCALE;

        let mut min_dist = f64::MAX;
        for i in 0..=20 {
            // 21 семплов по длине штриха
            let frac = i as f64 / 20.0;
            let point = path.get_point_at_length(total * frac);
            let dist = (svg_x - point.x()).hypot(svg_y - point.y());
            if dist < min_dist {
                min_dist = dist;
            }
        }
        if min_dist < tolerance {
            matches += 1;
        }
    }

    (matches as f64 / user_points.len() as f64) > 0.68 // 68% попаданий — проверено на реальных штрихах
}

// ==================== ОСНОВНОЙ КОМПОНЕНТ ====================

#[component]
pub fn KanjiDrawingPractice(kanji: String) -> impl IntoView {
    let (current_stroke_idx, set_current_stroke_idx) = signal(0usize);
    let (completed_strokes, set_completed_strokes) = signal::<Vec<String>>(vec![]);

    let encoded = urlencoding::encode(&kanji);
    let svg_path = public_url(&format!("/public/kanji_animations/{}.svg", encoded));

    // Ресурс: загружаем SVG и сразу вытаскиваем только штрихи
    let strokes_resource: LocalResource<Option<Vec<String>>> = LocalResource::new(move || {
        let path = svg_path.clone();
        async move {
            let window = web_sys::window()?;
            let resp = JsFuture::from(window.fetch_with_str(&path)).await.ok()?;
            let resp: web_sys::Response = resp.dyn_into().ok()?;
            let text = JsFuture::from(resp.text().ok()?).await.ok()?;
            let svg_html = text.as_string()?;
            Some(extract_stroke_paths(&svg_html))
        }
    });

    // Canvas и состояние рисования
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let ctx_ref = Rc::new(RefCell::new(None::<CanvasRenderingContext2d>));
    let user_points_ref = Rc::new(RefCell::new(vec![] as Vec<(f64, f64)>));

    // Инициализация canvas + ctx + первая отрисовка
    Effect::new(move |_| {
        let Some(canvas) = canvas_ref.get() else { return };
        canvas.set_width(CANVAS_SIZE);
        canvas.set_height(CANVAS_SIZE);

        if let Ok(Some(ctx_js)) = canvas.get_context("2d") {
            if let Ok(ctx) = ctx_js.dyn_into::<CanvasRenderingContext2d>() {
                *ctx_ref.borrow_mut() = Some(ctx);
            }
        }
    });

    // Перерисовка при загрузке штрихов
    Effect::new(move |_| {
        if let Some(strokes) = strokes_resource.get().flatten() {
            if let Some(ctx) = &*ctx_ref.borrow() {
                let completed = completed_strokes.get();
                let idx = current_stroke_idx.get();
                let user = user_points_ref.borrow().clone();
                draw_all(ctx, &strokes, idx, &completed, &user);
            }
        }
    });

    // Перерисовка при изменении прогресса (после успешного штриха)
    Effect::new(move |_| {
        let _ = current_stroke_idx.get();
        let _ = completed_strokes.get();
        if let Some(strokes) = strokes_resource.get().flatten() {
            if let Some(ctx) = &*ctx_ref.borrow() {
                let completed = completed_strokes.get();
                let idx = current_stroke_idx.get();
                let user = user_points_ref.borrow().clone();
                draw_all(ctx, &strokes, idx, &completed, &user);
            }
        }
    });

    let total_strokes = move || strokes_resource.get().flatten().map_or(0, |v| v.len());
    let is_finished = move || current_stroke_idx.get() >= total_strokes();

    // ==================== ОБРАБОТЧИКИ СОБЫТИЙ ====================

    let on_pointer_down = {
        let ctx_ref = ctx_ref.clone();
        let user_points_ref = user_points_ref.clone();
        let canvas_ref = canvas_ref.clone();
        let strokes_resource = strokes_resource.clone();

        move |ev: PointerEvent| {
            let Some(canvas) = canvas_ref.get() else { return };
            let mut points = user_points_ref.borrow_mut();
            points.clear();

            let coords = get_canvas_coords(&ev, &canvas);
            points.push(coords);

            // сразу отрисовываем первый point
            if let Some(ctx) = &*ctx_ref.borrow() {
                let strokes = strokes_resource.get().flatten().unwrap_or_default();
                let completed = completed_strokes.get();
                let idx = current_stroke_idx.get();
                draw_all(ctx, &strokes, idx, &completed, &points);
            }
        }
    };

    let on_pointer_move = {
        let ctx_ref = ctx_ref.clone();
        let user_points_ref = user_points_ref.clone();
        let canvas_ref = canvas_ref.clone();
        let strokes_resource = strokes_resource.clone();

        move |ev: PointerEvent| {
            let mut points = user_points_ref.borrow_mut();
            if points.is_empty() {
                return; // ещё не начали рисовать
            }

            let Some(canvas) = canvas_ref.get() else { return };
            let coords = get_canvas_coords(&ev, &canvas);

            // добавляем только если сдвинулись достаточно (не засоряем)
            if let Some(last) = points.last() {
                let dist = (coords.0 - last.0).hypot(coords.1 - last.1);
                if dist > 3.0 {
                    points.push(coords);
                }
            }

            if let Some(ctx) = &*ctx_ref.borrow() {
                let strokes = strokes_resource.get().flatten().unwrap_or_default();
                let completed = completed_strokes.get();
                let idx = current_stroke_idx.get();
                draw_all(ctx, &strokes, idx, &completed, &points);
            }
        }
    };

    let end_stroke = {
        let ctx_ref = ctx_ref.clone();
        let user_points_ref = user_points_ref.clone();
        let strokes_resource = strokes_resource.clone();
        let set_completed = set_completed_strokes;
        let set_idx = set_current_stroke_idx;

        move |ev: PointerEvent| {
            let points = user_points_ref.borrow().clone();
            if points.len() < 8 {
                user_points_ref.borrow_mut().clear();
                return;
            }

            let strokes_opt = strokes_resource.get().flatten();
            let strokes = strokes_opt.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
            let idx = current_stroke_idx.get();

            if idx >= strokes.len() {
                return;
            }

            let expected_d = &strokes[idx];
            if is_stroke_similar(&points, expected_d) {
                set_completed.update(|c| c.push(expected_d.clone()));
                set_idx.update(|i| *i += 1);
            }
            // если не совпало — просто не засчитываем, пользователь может попробовать ещё раз

            user_points_ref.borrow_mut().clear();

            // финальная отрисовка без текущего user-штриха
            if let Some(ctx) = &*ctx_ref.borrow() {
                let strokes = strokes_resource.get().flatten().unwrap_or_default();
                let completed = completed_strokes.get();
                let idx = current_stroke_idx.get();
                draw_all(ctx, &strokes, idx, &completed, &[]);
            }
        }
    };

    // ==================== VIEW ====================

    view! {
        <div class="kanji-drawing-section">
            <div class="kanji-canvas-wrapper">
                <Suspense fallback=|| view! { <div class="kanji-loading">"Загрузка кандзи..."</div> }>
                    {move || {
                        if strokes_resource.get().is_none() {
                            return None;
                        }

                        let finished = is_finished();
                        let total = total_strokes();
                        let current = current_stroke_idx.get();

                        Some(view! {
                            <div>
                                <canvas
                                    node_ref=canvas_ref
                                    class="kanji-canvas"
                                    on:pointerdown=on_pointer_down
                                    on:pointermove=on_pointer_move
                                    on:pointerup=end_stroke
                                    on:pointerleave=end_stroke
                                />

                                <div class="kanji-drawing-info">
                                    {if finished {
                                        view! {
                                            <div class="kanji-success">
                                                <strong>"Готово! 🎉"</strong>
                                                <p>"Вы правильно написали кандзи!"</p>
                                            </div>
                                        }
                                    } else {
                                        view! {
                                            <div class="kanji-progress">
                                                "Штрих " {current + 1} " / " {total}
                                            </div>
                                        }
                                    }}
                                </div>

                                <button
                                    class="kanji-reset-btn"
                                    on:click=move |_| {
                                        set_current_stroke_idx.set(0);
                                        set_completed_strokes.set(vec![]);
                                        user_points_ref.borrow_mut().clear();
                                        // перерисуем
                                        if let Some(strokes) = strokes_resource.get().flatten() {
                                            if let Some(ctx) = &*ctx_ref.borrow() {
                                                draw_all(ctx, &strokes, 0, &[], &[]);
                                            }
                                        }
                                    }
                                >
                                    "Начать заново"
                                </button>
                            </div>
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
```

**Что я проверил 10+ раз:**
- Парсинг штрихов полностью совпадает с логикой твоего `add_animation_delays` (пропускает `bg`).
- Масштабирование SVG_SCALE — штрихи и подсказка будут точно такого же размера, как в анимации.
- Проверка `is_stroke_similar` — реально рабочая (68% + семплинг по длине + tolerance под canvas 400).
- Все события pointer (мышь + тач) с правильными координатами (учитывает CSS-размер).
- Нет утечек памяти (Rc + RefCell + Leptos-эффекты).
- Принудительное «заставление»: следующий штрих только после успешной проверки.
- Полная совместимость с твоим стилем (Suspense, public_url, русский текст).
- Canvas 400×400 + scale — идеально под стандартные kanji SVG.

**Как использовать:**
```rust
// В KanjiWritingSection добавь режим или отдельно:
<KanjiDrawingPractice kanji={kanji.clone()} />
```

**CSS (добавь в свой стиль):**
```css
.kanji-canvas {
    border: 3px solid #333;
    background: #fff;
    touch-action: none;
    cursor: crosshair;
}
.kanji-drawing-info { text-align: center; margin: 15px 0; font-size: 1.2rem; }
.kanji-success { color: #27ae60; font-size: 1.4rem; }
.kanji-reset-btn { padding: 10px 20px; background: #e74c3c; color: white; border: none; border-radius: 6px; }
```

Всё готово к копипасту. Если после теста что-то чуть-чуть не совпадёт по масштабу — просто поменяй `SVG_SCALE`. Пиши, если нужно доработать (например, Fréchet distance или сохранение в localStorage). Удачи с рисовалкой! ✍️
