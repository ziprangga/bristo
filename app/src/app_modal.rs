// Design modal not yet implementation
// still using spawn osascript for help to Mac native dialog
use anyhow::Result;
use std::process::Command;

pub fn modal_process_kill_dialog(app_name: &str) -> Result<bool> {
    // AppleScript dialog with Yes/No buttons
    let script = format!(
        r#"
        display dialog "The app '{}' is still running.\nDo you want to kill its running process?\nBe careful to save your work first!" buttons {{"No", "Yes"}} default button "No"
        if button returned of result is "Yes" then
            return "YES"
        else
            return "NO"
        end if
        "#,
        app_name
    );

    let output = Command::new("osascript").arg("-e").arg(script).output()?;

    let response = String::from_utf8_lossy(&output.stdout);

    Ok(response.trim() == "YES")
}

// use objc2::ClassType;
// use objc2::msg_send;
// use objc2::rc::autoreleasepool;
// use objc2_app_kit::NSAlert;
// use objc2_foundation::NSString;

// pub fn modal_process_kill_dialog(app_name: &str) -> Result<bool, String> {
//     // autoreleasepool(|_pool| unsafe {
//     //     // Allocate + init NSAlert
//     //     let alert: *mut NSAlert = msg_send![NSAlert::class(), alloc];
//     //     let alert: *mut NSAlert = msg_send![alert, init];

//     //     // Set message text
//     //     let msg = NSString::from_str(&format!("The app '{}' is still running.", app_name));
//     //     let info = NSString::from_str(
//     //         "Do you want to kill its running process?\nBe careful to save your work first!",
//     //     );

//     //     let _: () = msg_send![alert, setMessageText: &*msg];
//     //     let _: () = msg_send![alert, setInformativeText: &*info];

//     //     // Add buttons
//     //     let yes = NSString::from_str("Yes");
//     //     let no = NSString::from_str("No");
//     //     let _: () = msg_send![alert, addButtonWithTitle: &*yes];
//     //     let _: () = msg_send![alert, addButtonWithTitle: &*no];

//     //     // Run modally
//     //     let response: i32 = msg_send![alert, runModal];

//     //     // 1000 = first button return (Yes)
//     //     response == 1000
//     // })
//     // =====================
//     let result = std::panic::catch_unwind(|| {
//         autoreleasepool(|_pool| unsafe {
//             // Allocate + init NSAlert
//             let alert: *mut NSAlert = msg_send![NSAlert::class(), alloc];
//             let alert: *mut NSAlert = msg_send![alert, init];

//             // Set message text
//             let msg = NSString::from_str(&format!("The app '{}' is still running.", app_name));
//             let info = NSString::from_str(
//                 "Do you want to kill its running process?\nBe careful to save your work first!",
//             );

//             let _: () = msg_send![alert, setMessageText: &*msg];
//             let _: () = msg_send![alert, setInformativeText: &*info];

//             // Add buttons
//             let yes = NSString::from_str("Yes");
//             let no = NSString::from_str("No");
//             let _: () = msg_send![alert, addButtonWithTitle: &*yes];
//             let _: () = msg_send![alert, addButtonWithTitle: &*no];

//             // Run modally
//             let response: i32 = msg_send![alert, runModal];

//             // 1000 = first button return (Yes)
//             response == 1000
//         })
//     });

//     result.map_err(|_| "Failed to display NSAlert dialog".to_string())
// }

// result:
// thread 'main' (65550) panicked at app/src/app_modal.rs:78:25:
// invalid message send to -[NSAlert addButtonWithTitle:]: expected return to have type code '@', but found 'v'
// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
