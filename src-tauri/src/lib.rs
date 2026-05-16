use rdev::{listen, Event, EventType};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

mod logger;
use logger::Logger;

mod selection;
use selection::get_selected_text;

#[derive(Debug, Clone)]
enum MouseEvent {
    Move(f64, f64),
    Down(String),
    Up(String),
    Wheel(i64),
}

#[tauri::command]
fn js_log(message: String) {
    println!("{}", message);
}

#[tauri::command]
async fn start_global_hook(app: AppHandle) {
    start_global_listener(app);
}

/// 复制文字到剪贴板
#[tauri::command]
fn copy_text(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("复制失败: {}", e))
}

/// 打开 URL
#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    // 使用 std::process::Command 来打开浏览器
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &url])
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    Ok(())
}

/// 获取当前鼠标位置（简化版本 - 使用上一次鼠标移动的位置）
static mut LAST_MOUSE_POS: (i32, i32) = (0, 0);

/// 关键：是否禁止发送 set_text 事件（直到下次鼠标按下）
static mut SUPPRESS_SET_TEXT: bool = false;

#[tauri::command]
fn get_mouse_position() -> Result<(i32, i32), String> {
    unsafe {
        if LAST_MOUSE_POS.0 == 0 && LAST_MOUSE_POS.1 == 0 {
            return Err("鼠标位置未初始化".to_string());
        }
        Ok(LAST_MOUSE_POS)
    }
}

/// 命令：用户关闭了工具栏，禁止发送 set_text 直到下次鼠标按下
#[tauri::command]
fn toolbar_closed() {
    unsafe {
        SUPPRESS_SET_TEXT = true;
        Logger::info("工具栏已关闭", &["禁止发送 set_text 直到下次鼠标按下"]);
    }
}

fn start_global_listener(app: AppHandle) {
    // 创建通道，用于从 rdev 回调发送鼠标事件
    let (tx, rx) = channel::<MouseEvent>();

    // 在单独线程中启动 rdev 监听
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let _ = listen(move |event: Event| {
            match event.event_type {
                EventType::MouseMove { x, y } => {
                    // 更新最后鼠标位置
                    unsafe {
                        LAST_MOUSE_POS = (x as i32, y as i32);
                    }
                    let _ = tx_clone.send(MouseEvent::Move(x, y));
                }
                EventType::ButtonPress(btn) => {
                    let name = format!("{:?}", btn);
                    let _ = tx_clone.send(MouseEvent::Down(name));
                }
                EventType::ButtonRelease(btn) => {
                    let name = format!("{:?}", btn);
                    let _ = tx_clone.send(MouseEvent::Up(name));
                }
                EventType::Wheel { delta_y, .. } => {
                    let _ = tx_clone.send(MouseEvent::Wheel(delta_y));
                }
                _ => {}
            }
        });
    });

    // 在主线程中处理鼠标事件
    let app_clone = app.clone();
    thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            match event {
                MouseEvent::Move(x, y) => {
                    println!("鼠标移动：({:.1}, {:.1})", x, y);
                    Logger::info("鼠标移动", &[&format!("({:.1}, {:.1})", x, y)]);
                    let _ = app_clone.emit("global_mouse_move", (x, y));
                }
                MouseEvent::Down(name) => {
                    Logger::info("鼠标按下", &[&name]);

                    // 关键：鼠标按下时重置禁止标志
                    unsafe {
                        if SUPPRESS_SET_TEXT {
                            SUPPRESS_SET_TEXT = false;
                            Logger::info("鼠标按下", &["重置 SUPPRESS_SET_TEXT"]);
                        }
                    }

                    let _ = app_clone.emit("global_mouse_down", name);
                }
                MouseEvent::Up(name) => {
                    Logger::info("鼠标松开", &[&name]);
                    let _ = app_clone.emit("global_mouse_up", name.clone());

                    // 延迟处理，确保选择完成
                    let app_for_selection = app_clone.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(200));

                        // 关键：检查是否禁止发送 set_text
                        unsafe {
                            if SUPPRESS_SET_TEXT {
                                Logger::info("鼠标松开", &["SUPPRESS_SET_TEXT 为 true，跳过显示"]);
                                return;
                            }
                        }

                        if let Some(text) = get_selected_text(&app_for_selection) {
                            Logger::info("选中的文字", &[&text]);

                            // 获取悬浮窗口并显示
                            if let Some(toolbar_window) =
                                app_for_selection.get_webview_window("word_highlight_assistant")
                            {
                                // 发送文字到悬浮窗口
                                let _ = app_for_selection.emit_to(
                                    "word_highlight_assistant",
                                    "set_text",
                                    text.clone(),
                                );

                                // 获取鼠标位置并移动窗口到鼠标附近
                                unsafe {
                                    let (x, y) = LAST_MOUSE_POS;
                                    let _ = toolbar_window.set_position(tauri::Position::Physical(
                                        tauri::PhysicalPosition {
                                            x: x - 175,
                                            y: y + 20,
                                        },
                                    ));
                                }

                                // 显示并聚焦窗口
                                let _ = toolbar_window.show();
                                let _ = toolbar_window.set_focus();
                            }
                        }
                    });
                }
                MouseEvent::Wheel(delta_y) => {
                    Logger::info("鼠标滚轮", &[&format!("{}", delta_y)]);
                    let _ = app_clone.emit("global_wheel", delta_y);
                }
            }
        }
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
        .invoke_handler(tauri::generate_handler![
            js_log,
            start_global_hook,
            copy_text,
            open_url,
            get_mouse_position,
            toolbar_closed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
