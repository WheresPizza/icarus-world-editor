/// Detect the ICARUS Dedicated Server executable via Steam library scan
pub fn detect_server_exe() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        detect_server_exe_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn detect_server_exe_windows() -> Option<String> {
    use std::path::PathBuf;

    // Try to get Steam install path from registry
    let steam_path = get_steam_install_path()?;

    // Parse libraryfolders.vdf to find all Steam library paths
    let library_folders_path = PathBuf::from(&steam_path)
        .join("steamapps")
        .join("libraryfolders.vdf");

    let library_paths = parse_library_folders(&library_folders_path);
    let mut all_paths = vec![steam_path];
    all_paths.extend(library_paths);

    // Check each library for ICARUS Dedicated Server (App ID 2089135)
    for lib_path in &all_paths {
        let manifest_path = PathBuf::from(lib_path)
            .join("steamapps")
            .join("appmanifest_2089135.acf");

        if manifest_path.exists() {
            let exe_path = PathBuf::from(lib_path)
                .join("steamapps")
                .join("common")
                .join("ICARUS Dedicated Server")
                .join("ICARUS")
                .join("Binaries")
                .join("Win64")
                .join("IcarusDedicatedServer-Win64-Shipping.exe");

            if exe_path.exists() {
                return Some(exe_path.to_string_lossy().to_string());
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn get_steam_install_path() -> Option<String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    // Read from HKCU\Software\Valve\Steam
    unsafe {
        use windows_sys::Win32::System::Registry::*;

        let key_name: Vec<u16> = "Software\\Valve\\Steam\0"
            .encode_utf16()
            .collect();
        let value_name: Vec<u16> = "InstallPath\0".encode_utf16().collect();

        let mut key_handle: isize = 0;
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            key_name.as_ptr(),
            0,
            KEY_READ,
            &mut key_handle,
        );

        if result != 0 {
            return None;
        }

        let mut buf = vec![0u16; 512];
        let mut buf_size = (buf.len() * 2) as u32;
        let mut reg_type: u32 = 0;

        let result = RegQueryValueExW(
            key_handle,
            value_name.as_ptr(),
            std::ptr::null_mut(),
            &mut reg_type,
            buf.as_mut_ptr() as *mut u8,
            &mut buf_size,
        );

        RegCloseKey(key_handle);

        if result != 0 {
            return None;
        }

        let len = (buf_size / 2) as usize;
        let s = OsString::from_wide(&buf[..len.saturating_sub(1)]);
        Some(s.to_string_lossy().to_string())
    }
}

fn parse_library_folders(path: &std::path::Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut paths = Vec::new();

    // Very simple VDF parser - look for "path" entries
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('"') {
            let parts: Vec<&str> = line.splitn(4, '"').collect();
            if parts.len() >= 4 && parts[1] == "path" {
                let path_val = parts[3];
                if !path_val.is_empty() {
                    paths.push(path_val.to_string());
                }
            }
        }
    }

    paths
}
