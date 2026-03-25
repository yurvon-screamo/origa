# Onboarding Page Fixes - Implementation Guide

## Overview

This document provides specific code fixes for the issues identified in the onboarding flow analysis.

---

## Issue 1: Event Bubbling in Checkbox Cards (CRITICAL)

**File**: `origa_ui/src/pages/onboarding/apps_step.rs`

**Problem**: Clicking the checkbox toggles state twice due to event bubbling.

**Solution Options**:

### Option A: Remove Card Click Handler (Recommended)

Remove the `on:click` handler from the card and only use checkbox interaction:

```rust
view! {
    <Card
        class=Signal::derive(move || {
            let base = "card card-shadow card-selectable";
            if is_selected.get() {
                format!("{} selected", base)
            } else {
                base.to_string()
            }
        })
        test_id=Signal::derive(move || app_test_id_for_card.clone())
    >
        <div class="flex items-center gap-4 p-2">
            <div class="text-3xl">{app_icon}</div>
            <div class="flex-1 cursor-pointer" on:click=move |_| { toggle_app.run(app_id_for_click.clone()); }>
                <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(move || format!("{}-name", app_test_id_for_name.clone()))>
                    {app_name}
                </Text>
                <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(move || format!("{}-desc", app_test_id_for_desc.clone()))>
                    {app_desc}
                </Text>
            </div>
            <Checkbox
                checked=Signal::derive(move || is_selected.get())
                label=Signal::derive(String::new)
                on_change=Callback::new(move |()| {
                    toggle_app.run(app_id_for_cb.clone());
                })
                test_id=Signal::derive(move || format!("{}-checkbox", app_test_id_for_checkbox.clone()))
            />
        </div>
    </Card>
}
```

### Option B: Add Stop Propagation (Checkbox Component)

Modify the checkbox component to stop event propagation:

**File**: `origa_ui/src/ui_components/checkbox.rs`

```rust
view! {
    <label class=move || format!("checkbox-container {}", checkbox_class.get()) data-testid=test_id_val>
        <input
            type="checkbox"
            checked=move || checkbox_checked.get()
            disabled=move || checkbox_disabled.get()
            on:change=move |ev| {
                // Stop event from bubbling to parent
                ev.stop_propagation();
                if let Some(cb) = on_change {
                    cb.run(());
                }
            }
            // Also stop click event
            on:click=move |ev| {
                ev.stop_propagation();
            }
        />
        <span class="checkbox-box"></span>
        <span>{move || checkbox_label.get()}</span>
    </label>
}
```

---

## Issue 2: Missing Dropdown test_ids (HIGH)

**Files**:

- `origa_ui/src/pages/onboarding/progress/migii_selector.rs`
- `origa_ui/src/pages/onboarding/progress/duolingo_selector.rs`
- `origa_ui/src/pages/onboarding/progress/minna_selector.rs`

**Problem**: Dropdowns are missing `test_id` props, making them untestable.

**Fix for migii_selector.rs**:

```rust
view! {
    <Card class=Signal::derive(|| "p-4".to_string())>
        <Text size=TextSize::Default variant=TypographyVariant::Primary>
            "Migii"
        </Text>

        <div class="mt-4 space-y-4">
            <div>
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Уровень"
                </Text>
                <div class="mt-2">
                    <Dropdown
                        _options=Signal::derive(move || level_items.clone())
                        _selected=selected_level_value
                        _placeholder=Signal::derive(|| "Выберите уровень".to_string())
                        test_id=Signal::derive(|| "migii-level-dropdown".to_string())  // ← ADD THIS
                    />
                </div>
            </div>

            <Show when=move || selected_level.get().is_some()>
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Урок"
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            _options=lesson_items
                            _selected=selected_lesson_value
                            _placeholder=Signal::derive(|| "Выберите урок".to_string())
                            test_id=Signal::derive(|| "migii-lesson-dropdown".to_string())  // ← ADD THIS
                        />
                    </div>
                </div>
            </Show>
            ...
        </div>
    </Card>
}
```

**Fix for duolingo_selector.rs**:

```rust
<Dropdown
    _options=Signal::derive(move || module_items.clone())
    _selected=selected_module_value
    _placeholder=Signal::derive(|| "Выберите модуль".to_string())
    test_id=Signal::derive(move || format!("{}-module-dropdown", app_id.clone()))  // ← ADD THIS
/>
...
<Dropdown
    _options=unit_items
    _selected=selected_unit_value
    _placeholder=Signal::derive(|| "Выберите раздел".to_string())
    test_id=Signal::derive(move || format!("{}-unit-dropdown", app_id.clone()))  // ← ADD THIS
/>
```

**Fix for minna_selector.rs**:

```rust
<Dropdown
    _options=Signal::derive(move || lesson_items.clone())
    _selected=selected_lesson_value
    _placeholder=Signal::derive(|| "Выберите урок".to_string())
    test_id=Signal::derive(move || format!("{}-lesson-dropdown", app_id.clone()))  // ← ADD THIS
/>
```

---

## Issue 3: Accordion Height Animation Issues (MEDIUM)

**File**: `origa_ui/src/pages/onboarding/summary_step.rs`

**Problem**: Fixed height calculation may cut off content.

**Better Solution**: Use CSS `max-height: fit-content` with transition:

```rust
// Replace the inline style with CSS class approach
<div class="accordion-content">
    <div class="accordion-body">
        ...
    </div>
</div>
```

**CSS** (add to your styles):

```css
.accordion-content {
    max-height: 0;
    overflow: hidden;
    transition: max-height 0.3s ease-out;
}

.accordion-item.active .accordion-content {
    max-height: fit-content; /* Modern browsers */
    /* Fallback for older browsers: */
    max-height: 1000px; /* Adjust based on expected max content */
}

.accordion-body {
    padding: 16px;
}
```

---

## Issue 4: Empty Checkbox Labels (LOW)

**Files**:

- `origa_ui/src/pages/onboarding/apps_step.rs:177`
- `origa_ui/src/pages/onboarding/summary_step.rs:189`

**Problem**: Empty labels reduce accessibility.

**Fix for apps_step.rs**:

```rust
<Checkbox
    checked=Signal::derive(move || is_selected.get())
    label=Signal::derive(move || app_name.clone())  // ← Use app name as label
    on_change=Callback::new(move |()| {
        toggle_app.run(app_id_for_cb.clone());
    })
    test_id=Signal::derive(move || format!("{}-checkbox", app_test_id_for_checkbox.clone()))
/>
```

**Fix for summary_step.rs**:

```rust
<Checkbox
    checked=Signal::derive(move || !is_excluded.get())
    label=Signal::derive(move || set_title.clone())  // ← Use set title as label
    on_change=Callback::new(move |()| {
        toggle_set.run(set_id_for_cb.clone());
    })
    test_id=Signal::derive(move || format!("{}-checkbox", set_test_id_1.clone()))
/>
```

**Update checkbox.rs to visually hide but keep accessible**:

```rust
view! {
    <label class=move || format!("checkbox-container {}", checkbox_class.get()) data-testid=test_id_val>
        <input
            type="checkbox"
            checked=move || checkbox_checked.get()
            disabled=move || checkbox_disabled.get()
            on:change=move |ev| {
                ev.stop_propagation();
                if let Some(cb) = on_change {
                    cb.run(());
                }
            }
            on:click=move |ev| {
                ev.stop_propagation();
            }
        />
        <span class="checkbox-box"></span>
        <span class="sr-only">{move || checkbox_label.get()}</span>  // ← Screen reader only
    </label>
}
```

**CSS**:

```css
.sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
}
```

---

## Issue 5: Card Selection Visual Feedback (MEDIUM)

**File**: `origa_ui/src/pages/onboarding/apps_step.rs:150-156`

**Problem**: Selected state styling may not work correctly.

**Current**:

```rust
class=Signal::derive(move || {
    let base = "card card-shadow card-selectable";
    if is_selected.get() {
        format!("{} selected", base)
    } else {
        base.to_string()
    }
})
```

**Improved**:

```rust
class=Signal::derive(move || {
    let mut classes = vec!["card", "card-shadow", "card-selectable"];
    if is_selected.get() {
        classes.push("card-selected");  // Use consistent naming
    }
    classes.join(" ")
})
```

**CSS**:

```css
.card-selectable {
    cursor: pointer;
    transition: all 0.2s ease;
    border: 2px solid transparent;
}

.card-selectable:hover {
    border-color: var(--color-olive-300);
    transform: translateY(-2px);
}

.card-selectable.card-selected {
    border-color: var(--color-olive-500);
    background-color: var(--color-olive-50);
}
```

---

## Issue 6: Import Error Handling (LOW)

**File**: `origa_ui/src/pages/onboarding/mod.rs:173-176`

**Problem**: Import errors are only logged, not shown to user.

**Fix**:

```rust
Err(e) => {
    tracing::error!("Import failed: {:?}", e);
    // Show error to user
    state.update(|s| {
        s.import_error = Some(format!("Ошибка импорта: {}", e));
    });
    is_importing.set(false);
}
```

**Add to OnboardingState** (in `onboarding_state.rs`):

```rust
pub struct OnboardingState {
    pub current_step: OnboardingStep,
    pub selected_level: Option<JapaneseLevel>,
    pub selected_apps: HashSet<String>,
    pub app_progress: HashMap<String, String>,
    pub sets_to_import: Vec<WellKnownSetMeta>,
    pub excluded_sets: HashSet<String>,
    pub available_sets: Vec<WellKnownSetMeta>,
    pub import_error: Option<String>,  // ← ADD THIS
}
```

**Display in view**:

```rust
<Show when=move || state.get().import_error.is_some()>
    <div class="error-message mt-4 p-4 bg-red-100 text-red-700 rounded">
        {move || state.get().import_error.clone().unwrap_or_default()}
    </div>
</Show>
```

---

## Issue 7: JLPT Step Missing test_ids (MEDIUM)

**File**: `origa_ui/src/pages/onboarding/jlpt_step.rs`

**Problem**: JLPT level options don't have test_ids.

**Fix**:

Add test_id prop to each JLPT level option:

```rust
// For each level button/option
<button
    class=move || format!("jlpt-option {}", if is_selected { "selected" } else { "" })
    data-testid=format!("jlpt-option-{}", level_code)  // ← ADD THIS
    on:click=move |_| select_level.run(level)
>
    {level_display}
</button>
```

---

## Summary of Changes Required

### High Priority (Blocking E2E Tests)

1. ✅ Add `test_id` props to all Dropdown components in progress selectors
2. ✅ Fix event bubbling in Checkbox component
3. ✅ Add `test_id` to JLPT level options

### Medium Priority (Improved UX)

1. ✅ Improve accordion height animation (CSS-based)
2. ✅ Add visual feedback for card selection
3. ✅ Add meaningful labels to checkboxes (for accessibility)

### Low Priority (Nice to Have)

1. ✅ Add import error handling with user feedback
2. ✅ Optimize Effect usage in progress selectors (performance)

---

## Testing After Fixes

Run these commands after applying fixes:

```bash
# Type check
cargo check -p origa_ui

# Build frontend
cd origa_ui && trunk build

# Run e2e tests
cd end2end
npm test
```
