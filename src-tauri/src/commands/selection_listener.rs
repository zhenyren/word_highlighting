use tauri::{Manager, Emitter};

const WINDOW_NAME: &str = "word_highlight_assistant";

#[tauri::command]
pub fn start_selection_listener() {
}

#[tauri::command]
pub fn stop_selection_listener() {
}

#[tauri::command]
pub fn show_text_in_window(app_handle: tauri::AppHandle, text: String) {
    let trimmed_text = text.trim();
    if !trimmed_text.is_empty() {
        if let Some(window) = app_handle.get_webview_window(WINDOW_NAME) {
            let _ = window.emit("selection-changed", trimmed_text);
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
