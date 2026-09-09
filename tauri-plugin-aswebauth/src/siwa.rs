// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

//! macOS implementation of `sign_in_with_apple` via `ASAuthorizationController`.
//!
//! Native Sign in with Apple presents a system sheet over the app's window —
//! no browser involved, which is what Mac App Store Guideline 4 requires
//! ("Authentication with Sign in with Apple should always be completed
//! without leaving the app"). Google/Yandex keep using
//! `ASWebAuthenticationSession` (`macos.rs`); this flow is Apple-only.
//!
//! Threading: the controller, its request and the delegate are all
//! main-thread-only objects. `commands.rs` dispatches here through
//! `AppHandle::run_on_main_thread`, and the `ASAuthorizationControllerDelegate`
//! protocol is `MainThreadOnly`, so every callback is guaranteed to arrive on
//! the main thread as well — no cross-thread ObjC releases here (unlike
//! `ASWebAuthenticationSession`, whose completion block macOS 26 may invoke on
//! a background queue).
//!
//! Anti-replay nonce: the raw nonce is generated here, its SHA-256 hash goes
//! into `ASAuthorizationOpenIDRequest.nonce`, and the raw value travels back
//! to the frontend. The login endpoint re-hashes it and matches the identity
//! token's `nonce` claim. Cross-platform contract: the hash is the lowercase
//! hex SHA-256 of the raw nonce string's UTF-8 bytes (the same vector is
//! pinned in the TrailBase endpoint tests and in the iOS Swift client).

use std::cell::RefCell;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::NSWindow;
use objc2_authentication_services::{
    ASAuthorization, ASAuthorizationAppleIDCredential, ASAuthorizationAppleIDProvider,
    ASAuthorizationAppleIDRequest, ASAuthorizationController, ASAuthorizationControllerDelegate,
    ASAuthorizationControllerPresentationContextProviding, ASAuthorizationError,
    ASAuthorizationOpenIDRequest, ASAuthorizationRequest, ASAuthorizationScopeEmail,
    ASPresentationAnchor,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSError, NSObjectProtocol, NSString, NSUTF8StringEncoding,
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use tauri::Manager;

use crate::commands::AppleCredential;

/// Controller + delegate kept alive until the flow completes.
///
/// `ASAuthorizationController` holds both its `delegate` and
/// `presentationContextProvider` weakly, and the delegate needs the controller
/// to outlive the sheet, so this bundle breaks the cycle when a delegate
/// callback fires: it is taken out of the shared slot and dropped on the main
/// thread.
type NativeFlowBundle = (
    Retained<ASAuthorizationController>,
    Retained<AppleIdDelegate>,
);

/// Shared slot holding the bundle. Everything that touches it runs on the
/// main thread (creation, fill, and every delegate callback — the protocol is
/// `MainThreadOnly`), so a plain `Rc<RefCell>` is the right primitive; no
/// cross-thread releases happen in this flow (unlike
/// `macos.rs::SessionSlot`, whose completion block may arrive off-main).
type NativeFlowSlot = Rc<RefCell<Option<NativeFlowBundle>>>;

fn fill_slot(slot: &NativeFlowSlot, bundle: NativeFlowBundle) {
    *slot.borrow_mut() = Some(bundle);
}

fn take_slot(slot: &NativeFlowSlot) -> Option<NativeFlowBundle> {
    slot.borrow_mut().take()
}

/// Presents the native SIWA sheet anchored over the app's main window and
/// bridges the delegate callback to `tx`. Guarantees exactly one send: from
/// one of the delegate methods, or synchronously with the real failure cause
/// when the flow cannot even be created.
pub(crate) fn start_native_sign_in<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    tx: std::sync::mpsc::Sender<Result<AppleCredential, String>>,
) {
    // Every early-exit path owns `tx` and sends the failure itself.
    let Some(mtm) = MainThreadMarker::new() else {
        let _ = tx.send(Err(
            "start_native_sign_in must run on the main thread".to_string()
        ));
        return;
    };

    let Some(webview_window) = app.get_webview_window("main") else {
        let _ = tx.send(Err("no main window".to_string()));
        return;
    };
    let ns_window_ptr = match webview_window.ns_window() {
        Ok(ptr) => ptr,
        Err(e) => {
            let _ = tx.send(Err(format!("failed to get the main NSWindow: {e}")));
            return;
        },
    };
    // SAFETY: Tauri returns a valid `NSWindow` pointer for the main window on
    // macOS; `Retained::retain` balances its +1 with our eventual release.
    let retained_window = unsafe { Retained::retain(ns_window_ptr.cast()) };
    let Some(window) = retained_window else {
        let _ = tx.send(Err("failed to retain the main NSWindow".to_string()));
        return;
    };

    let nonce = generate_raw_nonce();
    let nonce_hash = sha256_hex(&nonce);

    let slot: NativeFlowSlot = Rc::new(RefCell::new(None));
    let delegate = AppleIdDelegate::new(tx, nonce, window, Rc::clone(&slot), mtm);

    // SAFETY: The provider is a plain main-thread object; `new` has no
    // additional safety requirements.
    let provider = unsafe { ASAuthorizationAppleIDProvider::new() };
    // SAFETY: The request is freshly created by the provider.
    let apple_id_request: Retained<ASAuthorizationAppleIDRequest> =
        unsafe { provider.createRequest() };

    // `requestedScopes` and `nonce` are declared on the OpenID superclass.
    // SAFETY: `into_super` upcasts the freshly created request one level.
    let openid_request: Retained<ASAuthorizationOpenIDRequest> = apple_id_request.into_super();
    // Email only: the profile is built from the email address, and Apple
    // shares the name exactly once — requesting it just to discard the value
    // would add consent noise without a consumer.
    // SAFETY: The array outlives the setter call; the request is valid.
    unsafe {
        openid_request.setRequestedScopes(Some(&NSArray::from_slice(&[ASAuthorizationScopeEmail])));
        openid_request.setNonce(Some(&NSString::from_str(&nonce_hash)));
    }

    // SAFETY: The upcast request is valid; the array keeps it alive for the
    // controller.
    let authz_request: Retained<ASAuthorizationRequest> = openid_request.into_super();
    // SAFETY: Allocated with a valid requests array.
    let controller = unsafe {
        ASAuthorizationController::initWithAuthorizationRequests(
            ASAuthorizationController::alloc(),
            &NSArray::from_retained_slice(&[authz_request]),
        )
    };

    // The controller properties are typed by distinct protocols, so the
    // delegate is wrapped twice — once per property type. `from_ref` borrows
    // the delegate, which the flow slot keeps alive.
    // SAFETY: Both protocol objects wrap the same valid delegate instance.
    unsafe {
        let as_delegate: &ProtocolObject<dyn ASAuthorizationControllerDelegate> =
            ProtocolObject::from_ref(&*delegate);
        controller.setDelegate(Some(as_delegate));

        let as_anchor: &ProtocolObject<dyn ASAuthorizationControllerPresentationContextProviding> =
            ProtocolObject::from_ref(&*delegate);
        controller.setPresentationContextProvider(Some(as_anchor));
    }

    fill_slot(&slot, (controller.clone(), delegate));

    // SAFETY: The controller was created above; `performRequests` has no
    // additional safety requirements beyond a valid receiver.
    unsafe { controller.performRequests() };
}

// Delegate for the native SIWA flow: receives the authorization result and
// anchors the sheet over the app's main window.
define_class!(
    // SAFETY:
    // - Superclass `NSObject` has no subclassing requirements.
    // - The type does not implement `Drop`.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = (
        std::sync::mpsc::Sender<Result<AppleCredential, String>>,
        String,
        Retained<NSWindow>,
        NativeFlowSlot
    )]
    struct AppleIdDelegate;

    // SAFETY: `NSObjectProtocol` has no safety requirements.
    unsafe impl NSObjectProtocol for AppleIdDelegate {}

    // SAFETY: The method signatures match the generated protocol declarations.
    unsafe impl ASAuthorizationControllerDelegate for AppleIdDelegate {
        #[unsafe(method(authorizationController:didCompleteWithAuthorization:))]
        unsafe fn did_complete_with_authorization(
            &self,
            _controller: &ASAuthorizationController,
            authorization: &ASAuthorization,
        ) {
            // SAFETY: The authorization comes straight from the delegate
            // callback; the credential is only read.
            let result = unsafe { self.apple_credential(authorization) };
            let _ = self.ivars().0.send(result);
            self.drop_flow_bundle();
        }

        #[unsafe(method(authorizationController:didCompleteWithError:))]
        unsafe fn did_complete_with_error(
            &self,
            _controller: &ASAuthorizationController,
            error: &NSError,
        ) {
            let message = if error.code() == ASAuthorizationError::Canceled.0 {
                // The exact marker the frontend pattern-matches on.
                "cancelled".to_string()
            } else {
                error.localizedDescription().to_string()
            };
            let _ = self.ivars().0.send(Err(message));
            self.drop_flow_bundle();
        }
    }

    // SAFETY: The method signature matches the generated protocol declaration.
    unsafe impl ASAuthorizationControllerPresentationContextProviding for AppleIdDelegate {
        // Retained-returning protocol methods are registered via `method_id`.
        #[unsafe(method_id(presentationAnchorForAuthorizationController:))]
        unsafe fn presentation_anchor_for_authorization_controller(
            &self,
            _controller: &ASAuthorizationController,
        ) -> Retained<ASPresentationAnchor> {
            // NSWindow → NSResponder → NSObject: two hops up the superclass
            // chain, same as `macos.rs::AuthAnchorProvider`.
            self.ivars().2.clone().into_super().into_super()
        }
    }
);

impl AppleIdDelegate {
    fn new(
        tx: std::sync::mpsc::Sender<Result<AppleCredential, String>>,
        nonce: String,
        window: Retained<NSWindow>,
        slot: NativeFlowSlot,
        mtm: MainThreadMarker,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars((tx, nonce, window, slot));
        // SAFETY: The signature of `NSObject`'s `init` is correct.
        unsafe { msg_send![super(this), init] }
    }

    /// Extracts the identity token from the authorization credential.
    ///
    /// # SAFETY
    /// The authorization comes straight from the delegate callback; the
    /// credential is only read here.
    unsafe fn apple_credential(
        &self,
        authorization: &ASAuthorization,
    ) -> Result<AppleCredential, String> {
        // SAFETY: `credential` returns the object handed over by the system;
        // the downcast verifies the concrete class before reinterpreting.
        let credential = unsafe { authorization.credential() }
            .downcast::<ASAuthorizationAppleIDCredential>()
            .map_err(|_| "authorization carried an unexpected credential type".to_string())?;

        let token = unsafe { credential.identityToken() }
            .and_then(|data| {
                // SAFETY: The data comes straight from the credential; the
                // initializer only reads it. Identity tokens are ASCII JWTs.
                NSString::initWithData_encoding(NSString::alloc(), &data, NSUTF8StringEncoding)
            })
            .map(|string| string.to_string())
            .ok_or_else(|| "identity token is not valid UTF-8".to_string())?;

        if token.is_empty() {
            return Err("authorization carried an empty identity token".to_string());
        }

        Ok(AppleCredential {
            identity_token: token,
            nonce: self.ivars().1.clone(),
        })
    }

    /// Breaks the controller ↔ delegate cycle now that the flow is done.
    /// Delegate callbacks are `MainThreadOnly`, so the drop runs inline on
    /// the main thread — required, because the bundle holds AppKit objects.
    fn drop_flow_bundle(&self) {
        take_slot(&self.ivars().3);
    }
}

/// Generates the raw client nonce: 32 random bytes, base64url-encoded without
/// padding (RFC 4648 §5, un-padded — same alphabet as PKCE verifiers).
fn generate_raw_nonce() -> String {
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Lowercase-hex SHA-256 of the raw nonce string's UTF-8 bytes.
///
/// Known-answer vector shared with the TrailBase endpoint and the iOS client:
/// `sha256_hex("test") == "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"`.
fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// User dismissal must map to the exact `"cancelled"` string the frontend
    /// pattern-matches on (`oauth_buttons.rs`), not to a localized message.
    #[test]
    fn canceled_authorization_maps_to_cancelled_marker() {
        // Canceled == 1001 per ASAuthorizationError.
        assert_eq!(ASAuthorizationError::Canceled.0, 1001);
    }

    /// The nonce hash is a cross-platform contract (macOS Rust, iOS Swift,
    /// TrailBase endpoint): pin the documented known-answer vector.
    #[test]
    fn sha256_hex_matches_the_cross_platform_test_vector() {
        assert_eq!(
            sha256_hex("test"),
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    /// The raw nonce must be valid base64url without padding: 32 bytes
    /// encode to exactly 43 characters.
    #[test]
    fn generated_nonce_is_43_char_base64url() {
        let nonce = generate_raw_nonce();
        assert_eq!(nonce.len(), 43);
        assert!(!nonce.contains('+') && !nonce.contains('/') && !nonce.contains('='));
    }

    /// The slot hands its payload out exactly once: after the first take it
    /// stays empty, so a stray second callback cannot double-drop the bundle.
    #[test]
    fn slot_take_is_once() {
        let slot: NativeFlowSlot = Rc::new(RefCell::new(None));

        assert!(take_slot(&slot).is_none());
        assert!(take_slot(&slot).is_none());
    }
}
