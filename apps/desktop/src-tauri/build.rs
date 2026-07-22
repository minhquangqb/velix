use tauri_build::{Attributes, DefaultPermissionRule, InlinedPlugin};

fn main() {
    // `notify`/`set_badge` are the only commands reachable from remote platform
    // webviews, so they live in an inlined plugin rather than the app manifest:
    // an app manifest would put *every* app command under the ACL, including the
    // ones only the trusted `ui` webview calls.
    tauri_build::try_build(
        Attributes::new().plugin(
            "velix",
            InlinedPlugin::new()
                .commands(&["notify", "set_badge", "open_external"])
                .default_permission(DefaultPermissionRule::AllowAllCommands),
        ),
    )
    .expect("failed to run tauri-build");
}
