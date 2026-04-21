use std::io::{BufRead, BufReader};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::{ServerConfig, ServerState, ServerStatus};

/// Spawn the ICARUS Dedicated Server process
pub fn start_server(
    config: &ServerConfig,
    exe_path: &str,
    prospect_id: &str,
    server_state: Arc<Mutex<ServerState>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut state = server_state.lock().unwrap();
    if state.status == ServerStatus::Running || state.status == ServerStatus::Starting {
        return Err("Server is already running".to_string());
    }
    state.status = ServerStatus::Starting;
    state.log_lines.clear();
    drop(state);

    let mut args = vec![
        format!("-SteamServerName={}", config.server_name),
        format!("-Port={}", config.port),
        format!("-MaxPlayers={}", config.max_players),
    ];

    if !prospect_id.is_empty() {
        args.push(format!("-Prospect={}", prospect_id));
    }

    if let Some(ref pwd) = config.password {
        if !pwd.is_empty() {
            args.push(format!("-ServerPassword={}", pwd));
        }
    }

    let child = Command::new(exe_path)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start server: {}", e))?;

    let pid = child.id();

    {
        let mut state = server_state.lock().unwrap();
        state.status = ServerStatus::Running;
        state.pid = Some(pid);
        state.start_time = Some(Instant::now());
    }

    // Pipe stdout/stderr to Tauri events on a background thread
    let server_state_clone = Arc::clone(&server_state);
    let app_handle_clone = app_handle.clone();

    std::thread::spawn(move || {
        pipe_output(child, server_state_clone, app_handle_clone);
    });

    Ok(())
}

fn pipe_output(
    mut child: Child,
    server_state: Arc<Mutex<ServerState>>,
    app_handle: tauri::AppHandle,
) {
    use tauri::Emitter;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let state = Arc::clone(&server_state);
        let handle = app_handle.clone();

        std::thread::spawn(move || {
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        let _ = handle.emit(
                            "server://log",
                            serde_json::json!({
                                "line": l,
                                "level": "info"
                            }),
                        );
                        let mut state = state.lock().unwrap();
                        state.log_lines.push(l);
                        if state.log_lines.len() > 1000 {
                            state.log_lines.drain(0..100);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let state = Arc::clone(&server_state);
        let handle = app_handle.clone();

        std::thread::spawn(move || {
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        let _ = handle.emit(
                            "server://log",
                            serde_json::json!({
                                "line": l,
                                "level": "error"
                            }),
                        );
                        let mut state = state.lock().unwrap();
                        state.log_lines.push(format!("[ERR] {}", l));
                        if state.log_lines.len() > 1000 {
                            state.log_lines.drain(0..100);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // Wait for process to exit
    let _ = child.wait();

    let mut state = server_state.lock().unwrap();
    state.status = ServerStatus::Stopped;
    state.pid = None;
    state.start_time = None;
}

/// Stop the server by killing the process by PID
pub fn stop_server(server_state: Arc<Mutex<ServerState>>) -> Result<(), String> {
    let pid = {
        let state = server_state.lock().unwrap();
        state.pid
    };

    if let Some(pid) = pid {
        #[cfg(target_os = "windows")]
        {
            kill_process_windows(pid)?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            use std::process::Command as Cmd;
            let _ = Cmd::new("kill").arg(pid.to_string()).status();
        }

        let mut state = server_state.lock().unwrap();
        state.status = ServerStatus::Stopped;
        state.pid = None;
        state.start_time = None;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn kill_process_windows(pid: u32) -> Result<(), String> {
    unsafe {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, TerminateProcess, PROCESS_TERMINATE,
        };

        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle == 0 {
            return Err(format!("Failed to open process {}", pid));
        }
        let result = TerminateProcess(handle, 1);
        CloseHandle(handle);
        if result == 0 {
            return Err(format!("Failed to terminate process {}", pid));
        }
        Ok(())
    }
}
