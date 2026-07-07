use std::{fs, path::Path};

// https://github.com/tauri-apps/tauri/issues/13419#issuecomment-3398457618
// Fix `STATUS_ENTRYPOINT_NOT_FOUND` error on Windows when testing.
fn main() {
    set_app_version_env();

    #[expect(clippy::unwrap_used)]
    #[cfg(windows)]
    {
        let mut attributes = tauri_build::Attributes::new();
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        add_manifest();
        tauri_build::try_build(attributes).unwrap();
    }
    #[cfg(not(windows))]
    {
        tauri_build::build();
    }
}

#[expect(clippy::expect_used)]
fn set_app_version_env() {
    let tauri_config_path = Path::new("tauri.conf.json");
    println!("cargo:rerun-if-changed={}", tauri_config_path.display());

    let tauri_config =
        fs::read_to_string(tauri_config_path).expect("Failed to read tauri.conf.json during build");
    let tauri_config: serde_json::Value =
        serde_json::from_str(&tauri_config).expect("Failed to parse tauri.conf.json during build");
    let version = tauri_config
        .get("version")
        .and_then(serde_json::Value::as_str)
        .expect("tauri.conf.json must contain a string version field");

    println!("cargo:rustc-env=FOCUST_APP_VERSION={version}");
}

#[expect(clippy::unwrap_used, clippy::expect_used)]
#[cfg(windows)]
fn add_manifest() {
    use std::env;

    static WINDOWS_MANIFEST_FILE: &str = "windows-app-manifest.xml";

    let manifest = env::current_dir()
        .expect("Failed to get current directory during build")
        .join(WINDOWS_MANIFEST_FILE);

    println!("cargo:rerun-if-changed={}", manifest.display());
    // Embed the Windows application manifest file.
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
        manifest.to_str().unwrap()
    );
    // Turn linker warnings into errors.
    println!("cargo:rustc-link-arg=/WX");
}
