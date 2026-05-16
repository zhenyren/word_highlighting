use rdev::{listen, Event, EventType};
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};
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

/// 双击检测：程序启动时间
static mut START_TIME: Option<Instant> = None;

/// 双击检测：上次点击的毫秒数
static mut LAST_CLICK_MS: u64 = 0;

/// 双击间隔阈值（毫秒）
const DOUBLE_CLICK_GAP: u64 = 500;

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
    // 保存程序启动时间
    unsafe {
        if START_TIME.is_none() {
            START_TIME = Some(Instant::now());
        }
    }

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
                    // /println!("鼠标移动：({:.1}, {:.1})", x, y);
                    // Logger::info("鼠标移动", &[&format!("({:.1}, {:.1})", x, y)]);
                    let _ = app_clone.emit("global_mouse_move", (x, y));
                }
                MouseEvent::Down(name) => {
                    // Logger::info("鼠标按下", &[&name]);

                    // 关键：鼠标按下时重置禁止标志
                    unsafe {
                        if SUPPRESS_SET_TEXT {
                            SUPPRESS_SET_TEXT = false;
                            // Logger::info("鼠标按下", &["重置 SUPPRESS_SET_TEXT"]);
                        }
                    }

                    let _ = app_clone.emit("global_mouse_down", name);
                }
                MouseEvent::Up(name) => {
                    Logger::info("鼠标松开", &[&format!("按钮: {}", name)]);
                    let _ = app_clone.emit("global_mouse_up", name.clone());

                    // 【最简单的双击检测】
                    let should_show = unsafe {
                        if let Some(start) = START_TIME {
                            let now_ms = start.elapsed().as_millis() as u64;
                            let old_ms = LAST_CLICK_MS;
                            let diff = now_ms - old_ms;
                            LAST_CLICK_MS = now_ms;

                            Logger::info(
                                "双击检测",
                                &[&format!(
                                    "当前: {}ms, 上次: {}ms, 间隔: {}ms",
                                    now_ms, old_ms, diff
                                )],
                            );

                            // 如果间隔 <= 500ms，就显示（不管是不是第一次，第一次是 0 没关系）
                            diff <= DOUBLE_CLICK_GAP
                        } else {
                            Logger::info("双击检测", &["START_TIME 没初始化"]);
                            false
                        }
                    };

                    Logger::info("双击检测", &[&format!("是否显示: {}", should_show)]);

                    if !should_show {
                        continue; // 继续下一次事件循环！不是 return！
                    }

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
                            // Logger::info("选中的文字", &[&text]);

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

                                // 获取鼠标位置并移动窗口到鼠标附近（带边界检测）
                                unsafe {
                                    let (mouse_x, mouse_y) = LAST_MOUSE_POS;

                                    // 窗口尺寸
                                    let window_width = 350i32;
                                    let window_height = 40i32;
                                    let margin = 10i32; // 边距

                                    // 获取所有显示器信息,找到鼠标所在的显示器
                                    let monitors =
                                        app_for_selection.available_monitors().unwrap_or_default();

                                    // 调试：打印所有显示器信息
                                    for (i, m) in monitors.iter().enumerate() {
                                        let pos = m.position();
                                        let size = m.size();
                                        Logger::info(
                                            "显示器",
                                            &[&format!(
                                                "#{} 位置: ({}, {}), 尺寸: {}x{}",
                                                i, pos.x, pos.y, size.width, size.height
                                            )],
                                        );
                                    }

                                    let (screen_x, screen_y, screen_width, screen_height) =
                                        monitors
                                            .iter()
                                            .find_map(|m| {
                                                let pos = m.position();
                                                let size = m.size();

                                                // Windows: rdev 和 Tauri 都使用物理像素,直接比较
                                                // 检查鼠标是否在此显示器范围内
                                                if mouse_x >= pos.x
                                                    && mouse_x < pos.x + size.width as i32
                                                    && mouse_y >= pos.y
                                                    && mouse_y < pos.y + size.height as i32
                                                {
                                                    Some((
                                                        pos.x,
                                                        pos.y,
                                                        size.width as i32,
                                                        size.height as i32,
                                                    ))
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap_or_else(|| {
                                                // 如果找不到匹配的显示器,使用主显示器
                                                app_for_selection
                                                    .primary_monitor()
                                                    .ok()
                                                    .flatten()
                                                    .map(|m| {
                                                        let pos = m.position();
                                                        let size = m.size();
                                                        (
                                                            pos.x,
                                                            pos.y,
                                                            size.width as i32,
                                                            size.height as i32,
                                                        )
                                                    })
                                                    .unwrap_or((0, 0, 1920, 1080))
                                            });

                                    // 计算可用工作区（扣除任务栏）
                                    let work_area = toolbar_window
                                        .inner_size()
                                        .map(|s| (s.width as i32, s.height as i32))
                                        .unwrap_or((screen_width, screen_height));

                                    Logger::info(
                                        "显示器信息",
                                        &[&format!(
                                            "鼠标: ({}, {}), 屏幕: ({}, {}) {}x{}, 工作区: {:?}",
                                            mouse_x,
                                            mouse_y,
                                            screen_x,
                                            screen_y,
                                            screen_width,
                                            screen_height,
                                            work_area
                                        )],
                                    );

                                    // 水平方向: 检查右边是否够放
                                    let screen_right = screen_x + screen_width;
                                    let right_space = screen_right - mouse_x;

                                    let pos_x: i32;
                                    if right_space >= window_width + margin {
                                        // 右边够,放右边
                                        pos_x = mouse_x + margin;
                                        Logger::info("水平", &["右边够,放右边"]);
                                    } else {
                                        // 右边不够,贴右边
                                        pos_x = screen_right - window_width;
                                        Logger::info("水平", &["右边不够,贴右边"]);
                                    }

                                    // 垂直方向: 检查下边是否够放
                                    let screen_bottom = screen_y + screen_height;
                                    let bottom_space = screen_bottom - mouse_y;

                                    let pos_y: i32;
                                    if bottom_space >= window_height + margin {
                                        // 下边够,放下边
                                        pos_y = mouse_y + margin;
                                        Logger::info("垂直", &["下边够,放下边"]);
                                    } else {
                                        // 下边不够,贴下边
                                        pos_y = screen_bottom - window_height;
                                        Logger::info("垂直", &["下边不够,贴下边"]);
                                    }

                                    Logger::info(
                                        "窗口位置",
                                        &[&format!(
                                            "鼠标: ({}, {}), 最终: ({}, {})",
                                            mouse_x, mouse_y, pos_x, pos_y
                                        )],
                                    );

                                    let _ = toolbar_window.set_position(tauri::Position::Physical(
                                        tauri::PhysicalPosition { x: pos_x, y: pos_y },
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
                &[&"[双击触发] 全局划词监听已启动...", &window.label()],
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
