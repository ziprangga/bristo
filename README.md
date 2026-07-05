# [![Bristo Icon](doc/icon/Bristo_64.png)](doc/icon/Bristo_64.png) **Bristo**

## Screenshot

<img width="500" alt="bristo UI" src="doc/Bristo_1.png" /> <img width="500" alt="Bristo UI" src="doc/Bristo_2.png" />

---

About Bristo
Bristo is a lightweight macOS cleanup tool built with Rust and a simple GUI using **[iced](https://github.com/iced-rs/iced)**. It helps safely inspect and remove unnecessary application files, including protected locations, BOM logs, and app metadata.

This app was built using open-source components and inspired by privacy guides from **[Sunknudsen](https://github.com/sunknudsen/guides/tree/main/archive/how-to-clean-uninstall-macos-apps-using-appcleaner-open-source-alternative)**. Some scripts in Bristo adapt references from his guides, so a big thank you goes to him!

The main goal is to help my friends and beloved wife who rarely use the terminal on macOS. On top of that, this project also serves as a way for me to dive deeper into Rust.

#### "Inspect and compile it"

---

## Permissions and Privacy Notice for macOS

To perform app cleanup effectively, **Bristo** requires specific permissions when running on macOS. Below is a detailed explanation:

### Access Data from other Application
Bristo may prompt for Access Data from other Application because it uses system APIs to inspect or query other apps safely — for example, reading BOM logs, metadata, or identifying running processes.

- This prompt appears even if Bristo only reads harmless metadata — no actual control or modification of other apps is performed.

- Denying this permission does not prevent scanning or viewing files.

- You can skip granting this permission; any step that depends on it may need to be done manually.

**How to Grant Access Data from other Application (Optional)**
1. open System Preferences → Security & Privacy → Privacy → Automation.
2. Ensure **Bristo** is allowed.
3. Restart the application after granting this permission.

### Access to Full Disk (Full Disk Access Permission)
Bristo needs access to protected locations, such as ~/Library/Containers, which are restricted by macOS’s security system. This access allows Bristo to safely move files or folders to the Trash during cleanup operations.

This permission is required. 
Without it, Bristo cannot remove some files or folders inside these protected locations. You can skip granting this permission, but any restricted files will need to be deleted manually.

**How to Grant Full Disk Access**
1.Open System Preferences → Security & Privacy → Privacy → Full Disk Access.
2. Ensure **Bristo** in the list and enabled.
3. Restart the application after granting this permission.

#### Inspect the Code (Optional)
The source code shows which operations and locations may trigger these permissions for transparency and safety.

---

## How to Use Bristo

Bristo makes cleaning up applications simple and intuitive. Follow these steps:

### Selecting an Application

- **Drag & Drop**: Drag the application you want to clean into the Bristo window.
- **Export Bom Logs**: Export BOM log files (if available) for advanced inspection. These logs can be used for manual review or troubleshooting — you can also refer to Sunknudsen’s explanation for guidance.

---

### Displaying Related Files or Folders

Once an application is added, Bristo will display a list of related files or folders.

- **Delete All**: Click the **Move to Trash** button to move all files/folders to the Trash.

---

### Verifying Deleted Files

Files or folders moved to the **Trash** can be reviewed. If needed, you can restore them to their original location.

---

### Opening File/Folder Locations

To open the location of a file or folder:

- **click** on the item in the path list name.

---

### Searching for Log Files (BOM File Log)

Bristo can search for BOM log files to enable a more thorough cleanup if the application provides them.

- **Default Location**: Log files are saved to the **Desktop** by default, but you can change this location using the input field.

---

## License

Licensed under:

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

---

## Contributing

Contributions are welcome! If you'd like to improve Bristo or add new features, please open an issue or submit a pull request.

---

## Need Help?

If you experience issues or have questions, please check the **[Wiki](https://github.com/ziprangga/bristo/wiki)** or open an issue.
