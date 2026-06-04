fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rustc-env=ORIGA_LANDING_BASE_URL={}",
        std::env::var("ORIGA_LANDING_BASE_URL").unwrap_or_else(|_| "https://origa.app".to_string())
    );
}
