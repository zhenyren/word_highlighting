use tauri::Manager;

const WINDOW_NAME: &str = "word_highlight_assistant";
#[tauri::command]
pub fn show_word_highlight_assistant_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window(WINDOW_NAME) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn close_word_highlight_assistant_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window(WINDOW_NAME) {
        let _ = window.hide();
    }
}
