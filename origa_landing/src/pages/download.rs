use leptos::prelude::*;

use crate::components::seo::PageMeta;
use crate::content::Locale;

const GITHUB_RELEASES_URL: &str = "https://github.com/yurvon-screamo/origa/releases/latest";
const WEB_APP_URL: &str = "https://app.origa.jp";

#[component]
pub fn DownloadPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();

    view! {
        <PageMeta locale path="/download" title=c.download_meta_title description=c.download_meta_description/>

        // Hero
        <section class="download-hero">
            <h1 class="download-hero__title">{c.download_h1}</h1>
            <p class="download-hero__subtitle">{c.download_subtitle}</p>
        </section>

        // Primary: Windows
        <section class="download-primary">
            <div class="download-primary__card">
                <div class="download-primary__header">
                    <div class="download-icon download-icon--primary" aria-hidden="true">
                        <IconWindows />
                    </div>
                    <div>
                        <p class="download-platform__name">{c.download_windows}</p>
                        <p class="download-platform__formats">{c.download_windows_formats}</p>
                    </div>
                </div>
                <a href=GITHUB_RELEASES_URL class="btn btn-filled btn-lg download-primary__btn">
                    {c.download_button}
                    " →"
                </a>
            </div>
        </section>

        // Secondary: macOS, Linux, Android, iOS
        <section class="download-secondary">
            <div class="download-secondary__grid">
                <DownloadCard
                    icon=view! { <IconApple /> }.into_any()
                    name=c.download_macos
                    formats=c.download_macos_formats
                    href=GITHUB_RELEASES_URL
                    button_text=c.download_button
                />
                <DownloadCard
                    icon=view! { <IconLinux /> }.into_any()
                    name=c.download_linux
                    formats=c.download_linux_formats
                    href=GITHUB_RELEASES_URL
                    button_text=c.download_button
                />
                <DownloadCard
                    icon=view! { <IconAndroid /> }.into_any()
                    name=c.download_android
                    formats=c.download_android_formats
                    href=GITHUB_RELEASES_URL
                    button_text=c.download_button
                />
                // iOS — coming soon (no download button)
                <DownloadCardComingSoon
                    icon=view! { <IconApple /> }.into_any()
                    name=c.download_ios
                    formats=c.download_ios_formats
                    badge=c.download_ios_coming_soon
                />
            </div>
        </section>

        // Bottom CTA
        <section class="download-bottom">
            <p class="download-bottom__text">{c.download_not_ready}</p>
            <a href=WEB_APP_URL class="download-bottom__link">{c.download_try_web}</a>
        </section>
    }
}

#[component]
fn DownloadCard(
    #[prop(into)] icon: AnyView,
    name: &'static str,
    formats: &'static str,
    href: &'static str,
    button_text: &'static str,
) -> impl IntoView {
    view! {
        <div class="download-secondary__card">
            <div class="download-secondary__card-header">
                <div class="download-icon download-icon--secondary" aria-hidden="true">
                    {icon}
                </div>
                <div>
                    <p class="download-platform__name">{name}</p>
                    <p class="download-platform__formats">{formats}</p>
                </div>
            </div>
            <a href=href class="btn">{button_text}" →"</a>
        </div>
    }
}

#[component]
fn DownloadCardComingSoon(
    #[prop(into)] icon: AnyView,
    name: &'static str,
    formats: &'static str,
    badge: &'static str,
) -> impl IntoView {
    view! {
        <div class="download-secondary__card download-secondary__card--soon">
            <div class="download-secondary__card-header">
                <div class="download-icon download-icon--secondary" aria-hidden="true">
                    {icon}
                </div>
                <div>
                    <p class="download-platform__name">{name}</p>
                    <p class="download-platform__formats">{formats}</p>
                </div>
            </div>
            <p class="download-coming-soon-badge">{badge}</p>
        </div>
    }
}

#[component]
fn IconWindows() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="28" height="28">
            <path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4H10.949M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-12.9-1.801" />
        </svg>
    }
}

#[component]
fn IconApple() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="28" height="28">
            <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.8-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z" />
        </svg>
    }
}

#[component]
fn IconLinux() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="28" height="28">
            <path d="M12.504 0c-.155 0-.311.001-.466.004-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.562.562 0 00.039.155c-.031.074-.059.148-.085.223-.13.317-.28.623-.457.913-.195.298-.4.583-.523.909-.122.322-.133.677-.09 1.02.042.34.127.673.263.986.138.317.29.63.519.9.11.13.247.246.415.316.168.07.36.087.536.037.087-.024.17-.065.245-.118.074-.053.14-.118.195-.19.11-.143.18-.316.212-.498.063-.363.028-.734-.03-1.096-.06-.362-.142-.72-.165-1.082-.024-.36.008-.732.168-1.052.086-.168.214-.312.38-.39.082-.04.175-.06.268-.06.096 0 .192.022.28.065.167.082.293.229.376.398.158.325.184.7.155 1.06-.028.36-.11.717-.174 1.073-.064.357-.105.718-.048 1.07.028.175.083.346.176.496.047.074.102.143.168.201.065.058.141.106.224.137.17.063.36.056.531-.01.17-.066.313-.18.428-.322.233-.28.386-.62.519-.963.131-.34.229-.698.292-1.06.063-.363.09-.737.04-1.104-.048-.363-.178-.725-.428-.996-.06-.065-.126-.124-.197-.176.012-.067.02-.135.024-.203.042-.39-.023-.785-.177-1.144-.156-.36-.394-.677-.688-.92-.147-.12-.308-.222-.479-.302-.085-.04-.173-.075-.263-.103l-.053-.015c.076-.236.12-.485.128-.737.008-.253-.02-.508-.085-.753-.131-.488-.398-.926-.756-1.263-.18-.168-.38-.314-.596-.433-.216-.12-.447-.213-.687-.27-.24-.057-.49-.08-.735-.07-.123.006-.246.02-.366.044l-.045.01c-.075-.116-.16-.226-.253-.328-.186-.203-.41-.37-.66-.484-.25-.113-.527-.171-.803-.168z" />
        </svg>
    }
}

#[component]
fn IconAndroid() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="28" height="28">
            <path d="M17.523 15.341a.997.997 0 0 0 0-1.994.997.997 0 0 0 0 1.994m-11.046 0a.997.997 0 0 0 0-1.994.997.997 0 0 0 0 1.994m11.405-6.02l1.997-3.46a.416.416 0 0 0-.152-.567.416.416 0 0 0-.567.152l-2.022 3.5A12.16 12.16 0 0 0 12 8.075a12.16 12.16 0 0 0-5.138 1.17L4.84 5.746a.416.416 0 0 0-.567-.152.416.416 0 0 0-.152.567l1.997 3.46C2.688 11.186.344 14.663 0 18.766h24c-.344-4.103-2.688-7.58-6.118-9.445" />
        </svg>
    }
}
