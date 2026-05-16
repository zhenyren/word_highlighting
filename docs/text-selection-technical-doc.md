# 跨应用程序文字选择功能技术文档

## 1. 功能概述

本功能实现了在任意应用程序中选择文字后，自动捕获选中的文字并在 Tauri 应用中显示。支持所有支持 Ctrl+C 复制的应用程序，包括但不限于：

- 浏览器（Chrome、Edge、Firefox 等）
- Office 套件（Word、Excel、PowerPoint）
- 文本编辑器（VS Code、Notepad++、记事本等）
- PDF 阅读器
- 任何其他支持复制的应用程序

## 2. 实现原理

### 2.1 核心机制

采用 **"模拟复制 + 剪贴板读取"** 的方案：

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  1. 备份剪贴板   │────▶│  2. 清空剪贴板    │────▶│  3. 模拟 Ctrl+C │
└─────────────────┘     └──────────────────┘     └─────────────────┘
                                                           │
                                                           ▼
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  6. 恢复剪贴板   │────▶│  5. 读取剪贴板    │◀────│  4. 等待复制完成 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

### 2.2 流程说明

1. **备份剪贴板**：保存用户当前剪贴板的内容
2. **清空剪贴板**：确保能准确检测到新复制的内容
3. **模拟 Ctrl+C**：使用 `enigo` 库模拟键盘快捷键
4. **等待复制完成**：等待 150ms 确保复制操作完成
5. **读取剪贴板**：获取新复制的内容（即选中的文字）
6. **恢复剪贴板**：将原始剪贴板内容恢复，不影响用户正常使用

## 3. 核心代码说明

### 3.1 文字选择模块（selection.rs）

```rust
/// 获取当前选中的文字
pub fn get_selected_text(app: &AppHandle) -> Option<String> {
    // 1. 备份当前剪贴板内容
    let clipboard_backup: Option<String> = app.clipboard().read_text().ok();

    // 2. 清空剪贴板
    let _ = app.clipboard().clear();

    // 3. 等待一小段时间
    thread::sleep(Duration::from_millis(50));

    // 4. 模拟 Ctrl+C 复制
    let mut enigo = Enigo::new(&Settings::default()).ok()?;

    enigo.key(Key::Control, enigo::Direction::Press).ok()?;
    enigo.key(Key::Unicode('c'), enigo::Direction::Press).ok()?;
    enigo.key(Key::Unicode('c'), enigo::Direction::Release).ok()?;
    enigo.key(Key::Control, enigo::Direction::Release).ok()?;

    // 5. 等待复制完成
    thread::sleep(Duration::from_millis(150));

    // 6. 读取剪贴板
    let selected_text: Option<String> = app.clipboard().read_text().ok();

    // 7. 恢复原始剪贴板内容
    if let Some(backup) = clipboard_backup {
        let _ = app.clipboard().write_text(backup);
    } else {
        let _ = app.clipboard().clear();
    }

    // 8. 过滤无效内容
    selected_text.filter(|text| !text.trim().is_empty())
}
```

### 3.2 全局鼠标监听（lib.rs）

使用 **通道（Channel）模式** 解耦 `rdev` 回调和 Tauri 应用：

```rust
// 定义鼠标事件枚举
#[derive(Debug, Clone)]
enum MouseEvent {
    Move(f64, f64),
    Down(String),
    Up(String),
    Wheel(i64),
}

fn start_global_listener(app: AppHandle) {
    // 创建通道
    let (tx, rx) = channel::<MouseEvent>();

    // 在单独线程中启动 rdev 监听
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let _ = listen(move |event: Event| {
            match event.event_type {
                EventType::MouseMove { x, y } => {
                    let _ = tx_clone.send(MouseEvent::Move(x, y));
                }
                // ... 其他事件
                EventType::ButtonRelease(btn) => {
                    let name = format!("{:?}", btn);
                    let _ = tx_clone.send(MouseEvent::Up(name));
                }
                // ...
            }
        });
    });

    // 在主线程中处理鼠标事件
    let app_clone = app.clone();
    thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            match event {
                MouseEvent::Up(name) => {
                    let _ = app_clone.emit("global_mouse_up", name.clone());

                    // 延迟处理，获取选中的文字
                    let app_for_selection = app_clone.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(200));
                        if let Some(text) = get_selected_text(&app_for_selection) {
                            let _ = app_for_selection.emit("text_selected", text);
                        }
                    });
                }
                // ... 其他事件处理
            }
        }
    });
}
```

### 3.3 前端界面（App.tsx）

```typescript
import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

function App() {
  const [selectedText, setSelectedText] = useState<string>("");

  useEffect(() => {
    // 监听选中的文字事件
    const unlisten = listen<string>("text_selected", (event) => {
      setSelectedText(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <main className="container">
      <div>
        <h3>选中的文字:</h3>
        {selectedText ? (
          <div>{selectedText}</div>
        ) : (
          <div>在其他应用中选择文字后松鼠标...</div>
        )}
      </div>
    </main>
  );
}
```

## 4. 文件结构

```
src-tauri/
├── src/
│   ├── lib.rs              # 主入口，全局鼠标监听
│   ├── selection.rs        # 文字选择核心模块
│   ├── logger.rs           # 日志模块
│   └── main.rs             # 程序入口
├── Cargo.toml
└── ...

src/
├── App.tsx                 # 主界面，显示选中的文字
├── main.tsx                # React 入口
└── ...
```

## 5. 依赖项

### Rust 依赖（Cargo.toml）

```toml
[dependencies]
tauri = { version = "2", features = ["macos-private-api"] }
tauri-plugin-clipboard-manager = "2"
enigo = { version = "0.3.0", features = ["serde"] }
rdev = "0.5.3"
```

### 前端依赖

```json
{
  "@tauri-apps/api": "^2.0",
  "@tauri-apps/plugin-event": "^2.0"
}
```

## 6. 使用说明

### 6.1 启动应用

```bash
npm run tauri dev
```

### 6.2 使用步骤

1. 在任何应用程序中选中文字（拖拽鼠标）
2. 松开鼠标左键
3. 选中的文字会自动显示在 Tauri 应用界面中

### 6.3 支持的应用程序

所有支持 Ctrl+C 复制的应用程序，包括：

- Web 浏览器（Chrome、Edge、Firefox）
- Office 套件
- 文本编辑器
- PDF 阅读器
- 终端/命令行工具
- 等等

## 7. 注意事项

### 7.1 性能考虑

- 使用了 200ms 的延迟确保选择完成
- 使用线程休眠而非异步等待，避免占用 CPU
- 剪贴板操作有 50ms + 150ms 的等待时间

### 7.2 限制

- 仅在 Windows 平台测试（使用 `enigo` 的 Windows 键码）
- 需要应用程序支持 Ctrl+C 复制功能
- 某些安全软件可能会拦截模拟键盘输入
- 剪贴板内容会被临时替换（已自动恢复）

### 7.3 安全性

- 剪贴板内容在内存中处理，不会持久化存储
- 文字选择仅在本地进行，不上传到任何服务器
- 模拟键盘输入仅限于 Ctrl+C 组合键

## 8. 扩展建议

### 8.1 可能的改进方向

1. **热词检测**：识别特定词汇后自动触发动作
2. **翻译集成**：选中文字后自动显示翻译
3. **搜索引擎**：一键搜索选中的文字
4. **历史记录**：保存最近选中的文字列表
5. **快捷键支持**：添加自定义快捷键触发选择

### 8.2 跨平台支持

当前实现主要面向 Windows。如需支持 macOS 和 Linux：

- **macOS**: 使用 `objc` 和 `ApplicationServices` 框架
- **Linux**: 使用 `xclip` 或 `wl-clipboard` 工具

## 9. 参考资料

- [Tauri 官方文档](https://tauri.app/)
- [enigo 文档](https://github.com/enigo-rs/enigo)
- [rdev 文档](https://github.com/Narsil/rdev)
- [Rust 标准库 - 剪贴板操作](https://doc.rust-lang.org/std/)

---

**文档版本**: 1.0  
**最后更新**: 2025-07-16  
**作者**: AI Assistant
