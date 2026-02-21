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
) -> impl IntoView {
    view! {
        <div class=move || {
            let size_class = match size.get() {
                AvatarSize::Default => "avatar",
                AvatarSize::Small => "avatar-sm",
                AvatarSize::Large => "avatar-lg",
            };
            format!("{} {}", size_class, class.get())
        }>
            {move || initials.get()}
        </div>
    }
}

#[component]
pub fn AvatarGroup(children: Children) -> impl IntoView {
    view! {
        <div class="avatar-group">
            {children()}
        </div>
    }
}
