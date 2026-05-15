// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
use commands::{close_word_highlight_assistant_window, show_word_highlight_assistant_window};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            show_word_highlight_assistant_window,
            close_word_highlight_assistant_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
