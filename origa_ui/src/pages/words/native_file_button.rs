//! «Выбрать файл» — native desktop picker button.
//!
//! Desktop-Tauri only: on web and mobile the button does not render and the
//! WebView file input stays the single path. The host opens the dialog and
//! reads the picked file in one command (see `core/file_picker`), the bytes
//! are wrapped into a `File` and handed to the tab's existing callback —
//! zero changes in the pipelines.

use leptos::prelude::*;
use leptos::task::spawn_local;
use tracing::warn;

use crate::core::file_picker::{self, PickerKind};
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant};

#[component]
pub fn NativeFilePickerButton(
    kind: PickerKind,
    /// Reports whether the parent tab's scope is still alive; the async
    /// chain checks it before the single state write (`on_file`).
    disposed: Callback<(), bool>,
    on_file: Callback<web_sys::File>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        {move || {
            if !file_picker::is_available() {
                None
            } else {
                Some(view! {
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            spawn_local(async move {
                                let picked = file_picker::pick_and_read_file(kind).await;
                                if disposed.run(()) {
                                    return;
                                }
                                match picked {
                                    Ok(Some((name, bytes))) => {
                                        let mime = kind.mime_for_name(&name);
                                        match file_from_bytes(&name, bytes, mime) {
                                            Some(file) => on_file.run(file),
                                            None => warn!(name = %name, "Native pick: File construction failed"),
                                        }
                                    },
                                    Ok(None) => {}, // user cancelled
                                    Err(e) => warn!(error = %e, "Native file pick failed"),
                                }
                            });
                        })
                        test_id=test_id
                    >
                        {t!(i18n, common.choose_file)}
                    </Button>
                })
            }
        }}
    }
}

/// Wraps raw bytes into a `File` carrying the original name and MIME type.
/// `None` on construction failure (non-browser context / invalid name) —
/// the payload is dropped, the warning is the caller's.
fn file_from_bytes(name: &str, bytes: Vec<u8>, mime: &str) -> Option<web_sys::File> {
    let parts = js_sys::Array::new();
    let array = js_sys::Uint8Array::from(bytes.as_slice());
    parts.push(&array.into());
    let bag = web_sys::FilePropertyBag::new();
    web_sys::FilePropertyBag::set_type(&bag, mime);
    web_sys::File::new_with_str_sequence_and_options(&parts.into(), name, &bag).ok()
}

#[cfg(test)]
mod tests {
    use crate::core::file_picker::{PickerKind, native_picker_available};

    use rstest::rstest;

    #[rstest]
    #[case::web_browser(false, "web", false, false)]
    #[case::desktop_tauri_with_bridge(true, "windows", true, true)]
    #[case::desktop_tauri_macos(true, "macos", true, true)]
    #[case::desktop_tauri_linux(true, "linux", true, true)]
    #[case::android_tauri_no_native_picker(true, "android", true, false)]
    #[case::ios_tauri_no_native_picker(true, "ios", true, false)]
    #[case::desktop_tauri_without_bridge(true, "windows", false, false)]
    fn picker_visibility_follows_the_platform_matrix(
        #[case] is_tauri_shell: bool,
        #[case] platform: &str,
        #[case] dialog_bridge_present: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(
            native_picker_available(is_tauri_shell, platform, dialog_bridge_present),
            expected
        );
    }

    #[test]
    fn picker_kinds_carry_their_filters() {
        let (name, extensions) = PickerKind::Image.dialog_filter();
        assert_eq!(name, "Images");
        assert!(extensions.contains(&"png") && extensions.contains(&"jpg"));
        assert_eq!(PickerKind::AnkiDeck.dialog_filter().1, &["apkg"]);
    }

    /// The constructed File's type feeds the OCR data-URL parser — a wrong
    /// or empty type breaks the `data:image/` prefix even for valid bytes
    /// (empirically verified: FileReader derives the prefix from File.type).
    #[rstest]
    #[case::png("photo.png", "image/png")]
    #[case::jpg("photo.jpg", "image/jpeg")]
    #[case::jpeg("photo.JPEG", "image/jpeg")]
    #[case::webp("photo.webp", "image/webp")]
    #[case::unknown_extension("photo.heic", "")]
    fn image_mime_is_derived_from_the_file_name(#[case] name: &str, #[case] expected: &str) {
        assert_eq!(PickerKind::Image.mime_for_name(name), expected);
    }

    #[test]
    fn anki_mime_is_octet_stream_regardless_of_name() {
        assert_eq!(
            PickerKind::AnkiDeck.mime_for_name("deck.apkg"),
            "application/octet-stream"
        );
    }
}
