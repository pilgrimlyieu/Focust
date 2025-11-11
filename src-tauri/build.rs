// https://github.com/tauri-apps/tauri/issues/13419#issuecomment-3398457618
// Fix `STATUS_ENTRYPOINT_NOT_FOUND` error on Windows when testing.
fn main() {
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

#[cfg(windows)]
fn add_manifest() {
    static WINDOWS_MANIFEST_FILE: &str = "windows-app-manifest.xml";

    let manifest = std::env::current_dir()
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
