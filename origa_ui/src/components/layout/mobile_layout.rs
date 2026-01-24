use leptos::*;
use leptos_router::*;
use thaw::*;
use icondata::AiIcon;

#[component]
pub fn MobileLayout() -> impl IntoView {
    let is_drawer_open = create_rw_signal(false);
    
    view! {
        <div class="mobile-layout">
            <AppBar
                title="Origa"
                left_action=Some(AppBarAction {
                    icon: icondata::AiMenuOutlined,
                    on_click: Box::new(move |_| is_drawer_open.set(true)),
                })
                right_action=Some(AppBarAction {
                    icon: icondata::AiSettingOutlined,
                    on_click: Box::new(move |_| {
                        // Navigate to profile
                        leptos_router::use_navigate()("/profile", Default::default());
                    }),
                })
            />
            
            <NavDrawer
                open=is_drawer_open
                on_close=move |_| is_drawer_open.set(false)
                position="left"
            >
                <NavDrawerContent>
                    <div class="drawer-header">
                        <h3>"Origa"</h3>
                        <p>"Японский язык"</p>
                    </div>
                    <Divider />
                    <NavList>
                        <NavItem
                            value="overview"
                            href="/"
                            icon=icondata::AiDashboardOutlined
                            on_click=move |_| is_drawer_open.set(false)
                        >
                            "Обзор"
                        </NavItem>
                        <NavItem
                            value="learn"
                            href="/learn"
                            icon=icondata::AiBookOutlined
                            on_click=move |_| is_drawer_open.set(false)
                        >
                            "Учить"
                        </NavItem>
                        <NavItem
                            value="vocabulary"
                            href="/vocabulary"
                            icon=icondata::AiFontSizeOutlined
                            on_click=move |_| is_drawer_open.set(false)
                        >
                            "Словарь"
                        </NavItem>
                        <NavItem
                            value="kanji"
                            href="/kanji"
                            icon=icondata::AiFontSize
                            on_click=move |_| is_drawer_open.set(false)
                        >
                            "Кандзи"
                        </NavItem>
                        <NavItem
                            value="grammar"
                            href="/grammar"
                            icon=icondata::AiFileTextOutlined
                            on_click=move |_| is_drawer_open.set(false)
                        >
                            "Грамматика"
                        </NavItem>
                        <NavItem
                            value="grammar-reference"
                            href="/grammar-reference"
                            icon=icondata::AiBook
                            on_click=move |_| is_drawer_open.set(false)
                        >
                            "Справка"
                        </NavItem>
                        <NavItem
                            value="import"
                            href="/import"
                            icon=icondata::AiCloudUploadOutlined
                            on_click=move |_| is_drawer_open.set(false)
                        >
                            "Импорт"
                        </NavItem>
                        <NavItem
                            value="profile"
                            href="/profile"
                            icon=icondata::AiUserOutlined
                            on_click=move |_| is_drawer_open.set(false)
                        >
                            "Профиль"
                        </NavItem>
                    </NavList>
                </NavDrawerContent>
            </NavDrawer>

            <main class="main-content">
                <Outlet />
            </main>

            <BottomNavigation />
        </div>
    }
}

#[component]
pub fn BottomNavigation() -> impl IntoView {
    let navigate = leptos_router::use_navigate();
    let location = use_location();
    
    view! {
        <div class="bottom-nav">
            <button
                class="nav-item"
                class:active=move || location.pathname.get() == "/"
                on:click=move |_| navigate("/", Default::default())
            >
                <Icon icon=icondata::AiDashboardOutlined />
                <span>"Обзор"</span>
            </button>
            <button
                class="nav-item"
                class:active=move || location.pathname.get() == "/learn"
                on:click=move |_| navigate("/learn", Default::default())
            >
                <Icon icon=icondata::AiBookOutlined />
                <span>"Учить"</span>
            </button>
            <button
                class="nav-item"
                class:active=move || location.pathname.get().starts_with("/vocabulary")
                on:click=move |_| navigate("/vocabulary", Default::default())
            >
                <Icon icon=icondata::AiFontSizeOutlined />
                <span>"Словарь"</span>
            </button>
            <button
                class="nav-item"
                class:active=move || location.pathname.get().starts_with("/kanji")
                on:click=move |_| navigate("/kanji", Default::default())
            >
                <Icon icon=icondata::AiFontSize />
                <span>"Кандзи"</span>
            </button>
            <button
                class="nav-item"
                class:active=move || location.pathname.get() == "/profile"
                on:click=move |_| navigate("/profile", Default::default())
            >
                <Icon icon=icondata::AiUserOutlined />
                <span>"Профиль"</span>
            </button>
        </div>
    }
}