use enigo::{Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// 获取当前选中的文字
/// 原理：备份剪贴板 -> 模拟 Ctrl+C -> 读取剪贴板 -> 恢复剪贴板
pub fn get_selected_text(app: &AppHandle) -> Option<String> {
    // 1. 备份当前剪贴板内容
    // read_text 返回 Result<String, Error>，不是 Result<Option<String>, Error>
    let clipboard_backup: Option<String> = app.clipboard().read_text().ok();

    // 2. 清空剪贴板（确保能检测是否复制成功）
    let _ = app.clipboard().clear();

    // 3. 等待一小段时间确保剪贴板清空
    thread::sleep(Duration::from_millis(50));

    // 4. 模拟 Ctrl+C 复制
    let mut enigo = Enigo::new(&Settings::default()).ok()?;

    // 按下 Ctrl
    enigo.key(Key::Control, enigo::Direction::Press).ok()?;
    // 按下 C
    enigo.key(Key::Unicode('c'), enigo::Direction::Press).ok()?;
    // 松开 C
    enigo
        .key(Key::Unicode('c'), enigo::Direction::Release)
        .ok()?;
    // 松开 Ctrl
    enigo.key(Key::Control, enigo::Direction::Release).ok()?;

    // 5. 等待复制操作完成
    thread::sleep(Duration::from_millis(150));

    // 6. 读取剪贴板内容
    let selected_text: Option<String> = app.clipboard().read_text().ok();

    // 7. 恢复原始剪贴板内容
    if let Some(backup) = clipboard_backup {
        let _ = app.clipboard().write_text(backup);
    } else {
        let _ = app.clipboard().clear();
    }

    // 8. 过滤无效内容
    selected_text.filter(|text| {
        let trimmed = text.trim();
        // 排除空字符串和纯空白字符
        !trimmed.is_empty()
    })
}
