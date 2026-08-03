use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use cleaner;

#[test]
fn test_app_metadata_from_temp_path() -> cleaner::Result<()> {
    // Create temporary app folder
    let base_dir = std::env::temp_dir();
    let app_path = base_dir.join("test.app");
    fs::create_dir_all(app_path.join("Contents")).map_err(|e| {
        cleaner::ErrorKind::failed()
            .with_summary("Test setup failed")
            .with_reason(e.to_string())
    })?;

    // Create minimal Info.plist
    let plist_path = app_path.join("Contents/Info.plist");
    let mut plist_file = File::create(&plist_path).map_err(|e| {
        cleaner::ErrorKind::failed()
            .with_summary("Test setup failed")
            .with_reason(e.to_string())
    })?;

    // Minimal plist XML content
    let plist_content = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.example.test</string>
    <key>CFBundleExecutable</key>
        <string>test</string>
</dict>
</plist>
"#;
    plist_file
        .write_all(plist_content.as_bytes())
        .map_err(|e| {
            cleaner::ErrorKind::failed()
                .with_summary("Test setup failed")
                .with_reason(e.to_string())
        })?;

    // Now call your AppMetadata function
    let app_profile = cleaner::AppProfile::from_path(&app_path)?;
    assert_eq!(
        app_profile.as_app_metadata().as_bundle_id(),
        "com.example.test"
    );

    // Optional: clean up
    let _ = fs::remove_dir_all(&app_path);

    Ok(())
}

#[test]
fn test_running_processes_mock() -> cleaner::Result<()> {
    // Create temporary .app folder
    let base_dir = std::env::temp_dir();
    let app_path = base_dir.join("test.app");
    fs::create_dir_all(app_path.join("Contents")).map_err(|e| {
        cleaner::ErrorKind::failed()
            .with_summary("Test setup failed")
            .with_reason(e.to_string())
    })?;

    // Create minimal Info.plist
    let plist_path = app_path.join("Contents/Info.plist");
    let mut plist_file = File::create(&plist_path).map_err(|e| {
        cleaner::ErrorKind::failed()
            .with_summary("Test setup failed")
            .with_reason(e.to_string())
    })?;
    let plist_content = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.example.test</string>
    <key>CFBundleExecutable</key>
        <string>test</string>
</dict>
</plist>
"#;
    plist_file
        .write_all(plist_content.as_bytes())
        .map_err(|e| {
            cleaner::ErrorKind::failed()
                .with_summary("Test setup failed")
                .with_reason(e.to_string())
        })?;

    // Create AppProfile instance
    let app_profile = cleaner::AppProfile::from_path(&app_path)?;

    let mut cleaner = cleaner::Cleaner::new(app_profile);

    // Call find_app_processess; since nothing is really running, we just check it doesn't panic
    let _cleaner_ref = cleaner.find_app_process(None::<fn(std::borrow::Cow<'static, str>)>)?;
    // assert!(processes.is_empty());

    // Optional cleanup
    let _ = fs::remove_dir_all(&app_path);

    Ok(())
}

// Optional: test kill_processes
// Be careful: this can kill actual running processes, so usually skipped in automated tests
#[test]
#[ignore]
fn test_kill_processes_safe() -> cleaner::Result<()> {
    // let cleaner = cleaner::Cleaner::default();
    // Use a dummy .app path
    let app_path: PathBuf = PathBuf::from("/Applications/NonExistent.app");
    let app_metadata = cleaner::AppMetadata::new(
        app_path.clone(),
        "NonExistent.app".to_string(),
        "com.example.test".to_string(),
        "NonExistent".to_string(),
        "example".to_string(),
        "ex".to_string(),
    );
    let path_entry = cleaner::PathEntry::from_metadata(&app_metadata);

    let app_profile = cleaner::AppProfile::new(
        app_metadata,
        cleaner::AppProcs::default(),
        cleaner::AppLogReceipt::default(),
        path_entry,
    );

    let mut cleaner_instant = cleaner::Cleaner::new(app_profile);
    cleaner_instant.find_app_process(None::<fn(std::borrow::Cow<'static, str>)>)?;
    cleaner_instant.kill_app_process(None::<fn(usize, usize)>)?; // Safe: no processes exist
    Ok(())
}

#[test]
fn test_remove_child_when_parent_exists() {
    use std::path::PathBuf;

    let input = vec![
        (PathBuf::from("folderA/folderB"), "folderB".to_string()),
        (
            PathBuf::from("folderA/folderB/folderC"),
            "folderC".to_string(),
        ),
        (
            PathBuf::from("folderA/folderB/folderC/subX"),
            "subX".to_string(),
        ),
    ];

    // Simulate your filtering logic
    let mut sorted = input;
    sorted.sort_by_key(|(p, _)| p.components().count());

    let mut filtered: Vec<(PathBuf, String)> = Vec::new();

    'outer: for (path, name) in sorted {
        for (existing_path, _) in &filtered {
            if path.starts_with(existing_path) {
                continue 'outer;
            }
        }
        filtered.push((path, name));
    }

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, PathBuf::from("folderA/folderB"));
}
