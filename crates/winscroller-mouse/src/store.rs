//! Settings & saved slots persistence to `%APPDATA%\WinscrollerMouse\settings.json`.

use std::path::PathBuf;
use winscroller_mouse_core::settings::{AppStateFile, Settings};

pub fn app_data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("WinscrollerMouse")
}

pub fn local_data_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("WinscrollerMouse")
}

pub fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn load_state() -> AppStateFile {
    let path = settings_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(mut st) = serde_json::from_str::<AppStateFile>(&data) {
            st.ensure_slots();
            // Apply startup look if configured (1..12)
            if let Ok(idx) = st.settings.start_look.parse::<usize>()
                && idx >= 1
                && idx <= st.slots.len()
                && let Some(saved) = &st.slots[idx - 1]
            {
                let keep_start = st.settings.start_look.clone();
                st.settings = saved.clone();
                st.settings.start_look = keep_start;
            }
            return st;
        }
        // Fallback: legacy plain Settings file
        if let Ok(s) = serde_json::from_str::<Settings>(&data) {
            let mut st = AppStateFile {
                settings: s,
                slots: Vec::new(),
            };
            st.ensure_slots();
            return st;
        }
    }
    let mut st = AppStateFile::default();
    st.ensure_slots();
    st
}

pub fn save_state(st: &AppStateFile) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(st) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, &json).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        } else {
            let _ = std::fs::write(&path, json);
        }
    }
}
