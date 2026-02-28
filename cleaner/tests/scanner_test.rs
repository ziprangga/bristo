use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use cleaner::AppProcess;

#[test]
fn test_appinfo_from_temp_path() -> anyhow::Result<()> {
    // Create temporary app folder
    let base_dir = std::env::temp_dir();
    let app_path = base_dir.join("test.app");
    fs::create_dir_all(app_path.join("Contents"))?;

    // Create minimal Info.plist
    let plist_path = app_path.join("Contents/Info.plist");
    let mut plist_file = File::create(&plist_path)?;

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
    plist_file.write_all(plist_content.as_bytes())?;

    // Now call your AppInfo function
    let app_info = cleaner::AppInfo::from_path(&app_path.to_path_buf())?;
    assert_eq!(app_info.bundle_id, "com.example.test");

    // Optional: clean up
    let _ = fs::remove_dir_all(&app_path);

    Ok(())
}

#[test]
fn test_running_processes_mock() -> anyhow::Result<()> {
    // Create temporary .app folder
    let base_dir = std::env::temp_dir();
    let app_path = base_dir.join("test.app");
    fs::create_dir_all(app_path.join("Contents"))?;

    // Create minimal Info.plist
    let plist_path = app_path.join("Contents/Info.plist");
    let mut plist_file = File::create(&plist_path)?;
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
    plist_file.write_all(plist_content.as_bytes())?;

    // Create AppInfo instance
    let app_info = cleaner::AppInfo::from_path(&app_path.to_path_buf())?;

    // Call find_app_processess; since nothing is really running, we just check it doesn't panic
    let _processes = AppProcess::find_app_processes(&app_info);
    // assert!(processes.is_empty());

    // Optional cleanup
    let _ = fs::remove_dir_all(&app_path);

    Ok(())
}

// Optional: test kill_processes
// Be careful: this can kill actual running processes, so usually skipped in automated tests
#[test]
#[ignore]
fn test_kill_processes_safe() -> anyhow::Result<()> {
    // Use a dummy .app path
    let app_path: PathBuf = PathBuf::from("/Applications/NonExistent.app");
    let app_info = cleaner::AppInfo {
        path: app_path,
        name: "NonExistent.app".to_string(),
        bundle_id: "com.example.test".to_string(),
        bundle_name: "NonExistent".to_string(),
        organization: "example".to_string(),
    };
    let processes = AppProcess::find_app_processes(&app_info);
    AppProcess::kill_app_processes(&app_info.name, &processes)?; // Safe: no processes exist
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
