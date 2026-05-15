use tauri::Manager;

const WINDOW_NAME: &str = "word_highlight_assistant";
#[tauri::command]
pub fn show_word_highlight_assistant_window(app_handle: tauri::AppHandle) {
    let _ = tauri::WebviewWindowBuilder::new(
        &app_handle,
        WINDOW_NAME,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("单词助手")
    .inner_size(350.0, 40.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .shadow(true)
    .focused(true)
    .accept_first_mouse(true)
    .center()
    .build();
}

#[tauri::command]
pub fn close_word_highlight_assistant_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window(WINDOW_NAME) {
        let _ = window.close();
    }
}
