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
    #[prop(optional)] size: AvatarSize,
    #[prop(optional, into)] initials: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let size_class = match size {
        AvatarSize::Default => "avatar",
        AvatarSize::Small => "avatar-sm",
        AvatarSize::Large => "avatar-lg",
    };

    let full_class = format!("{} {}", size_class, class);

    view! {
        <div class=full_class>
            {initials}
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
