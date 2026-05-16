use rdev::{listen, Event, EventType};
use tauri::{AppHandle, Emitter, Manager};

mod logger;
use logger::Logger;

#[tauri::command]
fn js_log(message: String) {
    println!("{}", message);
}

#[tauri::command]
async fn start_global_hook(app: AppHandle) {
    start_global_listener(app);
}

fn start_global_listener(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let _ = listen(move |event: Event| match event.event_type {
            EventType::MouseMove { x, y } => {
                println!("鼠标移动：({:.1}, {:.1})", x, y);
                Logger::info("鼠标移动", &[&format!("({:.1}, {:.1})", x, y)]);
                let _ = app.emit("global_mouse_move", (x, y));
            }

            EventType::ButtonPress(btn) => {
                Logger::info("鼠标按下", &[&format!("{:?}", btn)]);
                let name = format!("{:?}", btn);
                let _ = app.emit("global_mouse_down", name);
            }

            EventType::ButtonRelease(btn) => {
                Logger::info("鼠标松开", &[&format!("{:?}", btn)]);
                let name = format!("{:?}", btn);
                let _ = app.emit("global_mouse_up", name);
            }

            EventType::Wheel { delta_y, .. } => {
                Logger::info("鼠标滚轮", &[&format!("{}", delta_y)]);
                let _ = app.emit("global_wheel", delta_y);
            }
            _ => {}
        });
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            Logger::success(
                "系统启动",
                &[&"[纯鼠标] 全局划词监听已启动...", &window.label()],
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![js_log, start_global_hook])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
