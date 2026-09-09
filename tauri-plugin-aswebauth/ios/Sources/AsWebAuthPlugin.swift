// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT
//
// Apple-platforms auth plugin for Tauri.
//
// 1. `startAuth`: ASWebAuthenticationSession wrapper for OAuth flows
//    (Google/Yandex). Replaces the tauri-plugin-opener + desktop-callback.html
//    + custom-scheme flow on iOS. ASWebAuthenticationSession opens Safari in
//    a dedicated auth context, intercepts the server-side 303 to
//    `origa://auth/callback` (redirect-initiated navigation — JavaScript hops
//    from an interstitial page are NOT intercepted), and returns the callback
//    URL via the completion handler — no CFBundleURLTypes needed.
//
//    `prefersEphemeralWebBrowserSession` is intentionally `false`: Google
//    OAuth rejects ephemeral (incognito-like) browser sessions, blocking
//    login.
//
// 2. `signInWithApple`: native Sign in with Apple via ASAuthorizationController
//    (Mac App Store Guideline 4 — Apple sign-in must complete without leaving
//    the app, no web involved). Returns the identity token and the raw
//    client nonce.

import AuthenticationServices
import SwiftRs
import Tauri
import UIKit
import WebKit

struct StartAuthArgs: Decodable {
    let url: String
    let callbackScheme: String
}

/// The Rust command generates the raw nonce and its lowercase-hex SHA-256
/// (see `tauri-plugin-aswebauth/src/nonce.rs` for the cross-platform
/// contract); this struct only carries the hash over to the sheet request.
struct SignInWithAppleArgs: Decodable {
    let nonceHash: String
    let nonce: String
}

class AsWebAuthPlugin: Plugin, ASWebAuthenticationPresentationContextProviding, ASAuthorizationControllerDelegate, ASAuthorizationControllerPresentationContextProviding {
    /// Strong reference to the active session. If this property is nil'd
    /// (or the plugin is deallocated) the session's completion handler
    /// will never fire, hanging the OAuth flow indefinitely.
    private var session: ASWebAuthenticationSession?

    /// Strong reference to the active native Sign in with Apple controller,
    /// plus the invoke and raw nonce of the flow it belongs to. The controller
    /// holds its delegate weakly, so clearing this property mid-flow would
    /// drop the callbacks and hang the invoke forever.
    private var appleSignInFlow: (invoke: Invoke, rawNonce: String)?
    private var authorizationController: ASAuthorizationController?

    @objc public func startAuth(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(StartAuthArgs.self)
        guard let url = URL(string: args.url) else {
            invoke.reject("Invalid URL")
            return
        }

        let session = ASWebAuthenticationSession(
            url: url,
            callbackURLScheme: args.callbackScheme
        ) { callbackURL, error in
            // Clear the strong reference so the session can be deallocated.
            self.session = nil

            if let error = error as? ASWebAuthenticationSessionError,
               error.code == .canceledLogin {
                invoke.reject("cancelled")
            } else if let error = error {
                invoke.reject("session failed: \(error.localizedDescription)")
            } else if let callbackURL = callbackURL {
                invoke.resolve(["url": callbackURL.absoluteString])
            } else {
                invoke.reject("no callback URL")
            }
        }

        session.presentationContextProvider = self
        // Google OAuth rejects ephemeral (incognito) sessions. Keep this false
        // so that Google and Yandex logins both work.
        session.prefersEphemeralWebBrowserSession = false

        self.session = session

        DispatchQueue.main.async {
            session.start()
        }
    }

    // MARK: - Native Sign in with Apple (ASAuthorizationController)

    /// Presents the system SIWA sheet — no browser involved (App Review
    /// Guideline 4: authentication must complete without leaving the app).
    ///
    /// Nonce contract shared with the macOS client and the TrailBase endpoint:
    /// the raw nonce is generated and hashed by the Rust command, which sends
    /// the hash here; this method puts it on the authorization request
    /// (`request.nonce`) and echoes the RAW value back with the result.
    /// Known-answer vector:
    /// sha256Hex("test") == "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".
    @objc public func signInWithApple(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SignInWithAppleArgs.self)

        // All flow-state mutations happen on the main queue: the delegate
        // callbacks (main-thread by protocol) clear the same properties, and
        // the invoke itself may arrive on a non-main IPC thread.
        DispatchQueue.main.async {
            guard self.appleSignInFlow == nil else {
                invoke.reject("a Sign in with Apple flow is already in progress")
                return
            }

            // Email only: the profile is built from the email address, and
            // Apple shares the name exactly once — requesting it just to
            // discard the value would add consent noise without a consumer.
            let request = ASAuthorizationAppleIDProvider().createRequest()
            request.requestedScopes = [.email]
            request.nonce = args.nonceHash

            let controller = ASAuthorizationController(authorizationRequests: [request])
            controller.delegate = self
            controller.presentationContextProvider = self

            self.appleSignInFlow = (invoke: invoke, rawNonce: args.nonce)
            self.authorizationController = controller

            controller.performRequests()
        }
    }

    private func takeAppleSignInFlow() -> (invoke: Invoke, rawNonce: String)? {
        let flow = appleSignInFlow
        appleSignInFlow = nil
        authorizationController = nil
        return flow
    }

    // MARK: - ASAuthorizationControllerDelegate

    func authorizationController(
        controller: ASAuthorizationController,
        didCompleteWithAuthorization authorization: ASAuthorization
    ) {
        guard let flow = takeAppleSignInFlow() else { return }

        guard
            let credential = authorization.credential as? ASAuthorizationAppleIDCredential,
            let tokenData = credential.identityToken,
            let identityToken = String(data: tokenData, encoding: .utf8),
            !identityToken.isEmpty
        else {
            flow.invoke.reject("authorization carried no identity token")
            return
        }

        flow.invoke.resolve([
            "identityToken": identityToken,
            "nonce": flow.rawNonce,
        ])
    }

    func authorizationController(
        controller: ASAuthorizationController,
        didCompleteWithError error: Error
    ) {
        guard let flow = takeAppleSignInFlow() else { return }

        if let authorizationError = error as? ASAuthorizationError,
           authorizationError.code == .canceled {
            // The exact marker the frontend pattern-matches on.
            flow.invoke.reject("cancelled")
        } else {
            flow.invoke.reject("apple sign-in failed: \(error.localizedDescription)")
        }
    }

    // MARK: - ASWebAuthenticationPresentationContextProviding

    func presentationAnchor(
        for session: ASWebAuthenticationSession
    ) -> ASPresentationAnchor {
        return Self.foregroundKeyWindowAnchor()
    }

    // MARK: - ASAuthorizationControllerPresentationContextProviding

    func presentationAnchor(
        for controller: ASAuthorizationController
    ) -> ASPresentationAnchor {
        return Self.foregroundKeyWindowAnchor()
    }

    /// The app's key window of the foreground-active scene, so auth UI
    /// appears on top of the WebView, not on a blank anchor.
    private static func foregroundKeyWindowAnchor() -> ASPresentationAnchor {
        let scenes = UIApplication.shared.connectedScenes
            .compactMap { $0 as? UIWindowScene }
            .filter { $0.activationState == .foregroundActive }

        for scene in scenes {
            for window in scene.windows where window.isKeyWindow {
                return window
            }
        }

        // Fallback: first window of the first scene.
        if let scene = scenes.first, let window = scene.windows.first {
            return window
        }

        // Last resort: empty anchor. The auth sheet may fail to display,
        // but this should be unreachable in a normal app lifecycle.
        return ASPresentationAnchor()
    }
}

@_cdecl("init_plugin_aswebauth")
func initPlugin() -> Plugin {
    return AsWebAuthPlugin()
}
