// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn show_mini_window(app_handle: tauri::AppHandle) {
    let _ = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "mini",
        tauri::WebviewUrl::App("index.html".into()), // 建议指向专门的路由或 HTML 文件
    )
    .title("小卡片")
    .inner_size(300.0, 40.0)
    .decorations(false) // 禁用边框和标题栏
    .transparent(true) // 必须为 true，否则圆角处会有黑色/白色底色
    .always_on_top(true)
    .shadow(true) // 开启阴影
    .center()
    .build();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, show_mini_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
