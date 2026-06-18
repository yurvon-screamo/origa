// TODO(TECH-DEBT): parameterize the production host (`app.origa.uwuwu.net`)
// via a build-time env var so it is declared in a single place. Currently the
// same literal appears in:
//   * tauri/tauri.conf.json  -> `security.csp` (`connect-src` + `img-src`)
//   * tauri/capabilities/default.json -> opener allow-list
//   * origa_ui/src/repository/trailbase_client.rs -> `env!("TRAILBASE_URL")`
//   * origa_ui/src/pages/login/oauth_buttons.rs -> `redirect_uri` builder
// Any host rename requires touching all four files manually, which is brittle.
fn main() {
    tauri_build::build()
}
