use leptos::html::Div;
use leptos::prelude::*;

const VIEWPORT_MARGIN: f64 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TooltipPlacement {
    #[default]
    Top,
    Bottom,
}

/// Pure placement decision. Prefers Top when there is enough room above
/// the trigger (`trigger_top >= tooltip_height + margin`); otherwise
/// flips to Bottom when the viewport offers more room below the trigger
/// than above. Boundary is non-strict (>=) so the tooltip does not
/// thrash between sides at the exact threshold.
pub(crate) fn decide_placement(
    trigger_top: f64,
    viewport_height: f64,
    tooltip_height: f64,
    margin: f64,
) -> TooltipPlacement {
    let needed = tooltip_height + margin;
    let room_above = trigger_top;
    let room_below = (viewport_height - trigger_top).max(0.0);

    if room_above >= needed {
        TooltipPlacement::Top
    } else if room_below > room_above {
        TooltipPlacement::Bottom
    } else {
        TooltipPlacement::Top
    }
}

#[component]
pub fn Tooltip(
    #[prop(optional, into)] text: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let placement = RwSignal::new(TooltipPlacement::default());
    let container_ref: NodeRef<Div> = NodeRef::new();
    let tooltip_ref: NodeRef<Div> = NodeRef::new();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let placement_class = move || match placement.get() {
        TooltipPlacement::Top => "tooltip tooltip--top",
        TooltipPlacement::Bottom => "tooltip tooltip--bottom",
    };

    let on_enter = move |_: leptos::ev::PointerEvent| {
        let Some(container) = container_ref.get() else {
            return;
        };
        let trigger_rect = container.get_bounding_client_rect();
        let tooltip_height = tooltip_ref
            .get()
            .map(|el| el.get_bounding_client_rect().height())
            .unwrap_or(40.0);
        let viewport_height = window()
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(800.0);
        placement.set(decide_placement(
            trigger_rect.top(),
            viewport_height,
            tooltip_height,
            VIEWPORT_MARGIN,
        ));
    };

    view! {
        <div
            class="tooltip-container"
            data-testid=test_id_val
            node_ref=container_ref
            on:pointerenter=on_enter
        >
            {children()}
            <div class=placement_class node_ref=tooltip_ref>
                {move || text.get()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decide_placement_near_top_returns_bottom() {
        assert_eq!(
            decide_placement(10.0, 800.0, 100.0, 8.0),
            TooltipPlacement::Bottom,
        );
    }

    #[test]
    fn decide_placement_mid_viewport_returns_top() {
        assert_eq!(
            decide_placement(400.0, 800.0, 100.0, 8.0),
            TooltipPlacement::Top,
        );
    }

    #[test]
    fn decide_placement_at_boundary_returns_top() {
        assert_eq!(
            decide_placement(108.0, 800.0, 100.0, 8.0),
            TooltipPlacement::Top,
        );
    }

    #[test]
    fn decide_placement_just_below_boundary_returns_bottom() {
        assert_eq!(
            decide_placement(107.0, 800.0, 100.0, 8.0),
            TooltipPlacement::Bottom,
        );
    }

    #[test]
    fn decide_placement_tight_viewport_picks_wider_side() {
        // Neither side fits the tooltip (room_above=10, room_below=50,
        // needed=108), so the function picks the side with more room — Bottom.
        assert_eq!(
            decide_placement(10.0, 60.0, 100.0, 8.0),
            TooltipPlacement::Bottom,
        );
    }
}
