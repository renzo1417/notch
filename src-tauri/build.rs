fn main() {
    // Share a single Common Controls v6 manifest across GNU debug app/test
    // targets. Tauri's default manifest otherwise covers app binaries only.
    if std::env::var("TARGET").is_ok_and(|target| target.ends_with("windows-gnu"))
        && std::env::var("PROFILE").as_deref() == Ok("debug")
    {
        let attributes = tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        tauri_build::try_build(attributes).expect("could not build Windows resources");
        embed_resource::compile_for_everything("windows-test-manifest.rc", embed_resource::NONE)
            .manifest_required()
            .expect("could not embed the Windows test manifest");
    } else {
        tauri_build::build();
    }
}
