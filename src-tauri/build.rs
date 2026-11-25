fn main() {
    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("resources/app.manifest"));
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
