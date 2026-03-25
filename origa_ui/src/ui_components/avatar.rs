use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum AvatarSize {
    #[default]
    Default,

    Small,

    Large,
}

#[component]
pub fn Avatar(
    #[prop(optional, into)] size: Signal<AvatarSize>,
    #[prop(optional, into)] initials: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div
            class=move || {
                let size_class = match size.get() {
                    AvatarSize::Default => "avatar",
                    AvatarSize::Small => "avatar-sm",
                    AvatarSize::Large => "avatar-lg",
                };
                format!("{} {}", size_class, class.get())
            }
            data-testid=test_id_val
        >
            {move || initials.get()}
        </div>
    }
}

#[component]
pub fn AvatarGroup(
    #[prop(optional, into)] test_id: Signal<String>,
    _children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class="avatar-group" data-testid=test_id_val>
            {_children()}
        </div>
    }
}
