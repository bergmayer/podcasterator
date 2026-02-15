#[cfg(target_os = "macos")]
pub fn pick_files_or_folder() -> Vec<String> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSModalResponseOK, NSOpenPanel};
    use objc2_foundation::{NSArray, NSString};

    // Must be called from the main thread
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let panel = NSOpenPanel::openPanel(mtm);

    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(true);
    panel.setAllowsMultipleSelection(true);

    // Filter to audio file types when selecting files
    let exts: Vec<_> = ["mp3", "m4a", "mp4", "m4b"]
        .iter()
        .map(|e| NSString::from_str(e))
        .collect();
    let array = NSArray::from_retained_slice(&exts);
    #[allow(deprecated)]
    panel.setAllowedFileTypes(Some(&array));
    // Allow selecting folders even though file types are filtered
    panel.setTreatsFilePackagesAsDirectories(false);

    let response = panel.runModal();
    if response != NSModalResponseOK {
        return Vec::new();
    }

    let urls = panel.URLs();
    urls.iter()
        .filter_map(|url| url.path().map(|p| p.to_string()))
        .collect()
}

#[cfg(not(target_os = "macos"))]
pub fn pick_files_or_folder() -> Vec<String> {
    Vec::new()
}
