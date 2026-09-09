// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &["start_auth", "sign_in_with_apple"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();
}
