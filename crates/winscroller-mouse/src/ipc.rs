//! Named-Pipe CLI & Single-Instance IPC (`\\.\pipe\WinscrollerMouse`).
//! Matches `omarchy-plume`'s CLI commands:
//!   toggle | on | off | find | look <name> | nextLook | undo | cursor <name> |
//!   set <key> <val> | save <1..12> | saveNew | load <1..12> | exportLook |
//!   importLook <json> | status | settings | quit

use std::sync::mpsc::Sender;
use windows::{
    Win32::{Foundation::*, Storage::FileSystem::*, System::Pipes::*},
    core::*,
};

const PIPE_NAME: PCWSTR = w!(r"\\.\pipe\WinscrollerMouse");

pub struct IpcRequest {
    pub command: String,
    pub reply: std::sync::mpsc::SyncSender<String>,
}

/// Try sending a CLI command to an already-running Winscroller Mouse instance.
/// Returns `Some(response)` if an instance was running and answered, or `None` if not running.
pub fn send_cli_command(args: &[String]) -> Option<String> {
    let payload = args.join("\t");
    unsafe {
        let handle = CreateFileW(
            PIPE_NAME,
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
        .ok()?;

        let mut written = 0u32;
        let _ = WriteFile(handle, Some(payload.as_bytes()), Some(&mut written), None);
        let _ = FlushFileBuffers(handle);

        let mut buf = vec![0u8; 8192];
        let mut read = 0u32;
        let ok = ReadFile(handle, Some(&mut buf), Some(&mut read), None).is_ok();
        let _ = CloseHandle(handle);

        if ok {
            Some(String::from_utf8_lossy(&buf[..read as usize]).to_string())
        } else {
            Some("ok".into())
        }
    }
}

/// Spawn a lightweight background thread listening on `\\.\pipe\WinscrollerMouse`.
/// Uses near-zero CPU/RAM (blocks cleanly inside kernel `ConnectNamedPipe`).
pub fn spawn_ipc_server(tx: Sender<IpcRequest>) {
    std::thread::Builder::new()
        .name("winscroller-ipc".into())
        .stack_size(256 * 1024) // minimal 256 KB stack for low memory!
        .spawn(move || {
            loop {
                unsafe {
                    let pipe = CreateNamedPipeW(
                        PIPE_NAME,
                        PIPE_ACCESS_DUPLEX,
                        PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                        4,
                        8192,
                        8192,
                        0,
                        None,
                    );
                    if pipe.is_invalid() {
                        std::thread::sleep(std::time::Duration::from_millis(250));
                        continue;
                    }
                    let connected = ConnectNamedPipe(pipe, None).is_ok()
                        || GetLastError() == ERROR_PIPE_CONNECTED;
                    if connected {
                        let mut buf = [0u8; 4096];
                        let mut read = 0u32;
                        if ReadFile(pipe, Some(&mut buf), Some(&mut read), None).is_ok() && read > 0
                        {
                            let cmd = String::from_utf8_lossy(&buf[..read as usize]).to_string();
                            let (rtx, rrx) = std::sync::mpsc::sync_channel(1);
                            if tx
                                .send(IpcRequest {
                                    command: cmd,
                                    reply: rtx,
                                })
                                .is_ok()
                            {
                                let resp = rrx
                                    .recv_timeout(std::time::Duration::from_secs(3))
                                    .unwrap_or_else(|_| "timeout".into());
                                let mut written = 0u32;
                                let _ = WriteFile(
                                    pipe,
                                    Some(resp.as_bytes()),
                                    Some(&mut written),
                                    None,
                                );
                                let _ = FlushFileBuffers(pipe);
                            }
                        }
                        let _ = DisconnectNamedPipe(pipe);
                    }
                    let _ = CloseHandle(pipe);
                }
            }
        })
        .ok();
}
