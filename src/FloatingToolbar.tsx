import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

// 当前窗口
const appWindow = getCurrentWebviewWindow();

function FloatingToolbar() {
  const [selectedText, setSelectedText] = useState<string>("");
  const [isVisible, setIsVisible] = useState<boolean>(false);
  // 关键：是否禁止显示（直到下次鼠标按下）
  const suppressUntilNextMouseDown = useRef<boolean>(false);

  useEffect(() => {
    // 监听 set_text 事件
    const unlisten = listen<string>("set_text", (event) => {
      // 如果被禁止显示，则忽略
      if (suppressUntilNextMouseDown.current) {
        console.log("显示被禁止，忽略 set_text 事件");
        return;
      }
      
      const text = event.payload;
      setSelectedText(text);
      setIsVisible(true);
    });

    // 关键：鼠标按下时重置禁止标志
    const unlistenMouseDown = listen("global_mouse_down", () => {
      if (suppressUntilNextMouseDown.current) {
        console.log("鼠标按下，重置禁止标志");
        suppressUntilNextMouseDown.current = false;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenMouseDown.then((fn) => fn());
    };
  }, []);

  // 复制文字到剪贴板
  const handleCopy = async () => {
    if (!selectedText) return;
    try {
      await invoke("copy_text", { text: selectedText });
      await handleClose();
    } catch (e) {
      console.error("复制失败:", e);
    }
  };

  // 百度搜索
  const handleSearch = async () => {
    if (!selectedText) return;
    try {
      const searchUrl = `https://www.baidu.com/s?wd=${encodeURIComponent(selectedText)}`;
      await invoke("open_url", { url: searchUrl });
      await handleClose();
    } catch (e) {
      console.error("搜索失败:", e);
    }
  };

  // 关闭窗口 - 关键：通知后端禁止显示
  const handleClose = async () => {
    setIsVisible(false);
    await appWindow.hide();
    
    // 关键：通知后端禁止发送 set_text 直到下次鼠标按下
    await invoke("toolbar_closed");
    
    console.log("窗口已关闭，后端已禁止显示直到下次鼠标按下");
  };

  if (!isVisible) {
    return null;
  }

  return (
    <div
      data-tauri-drag-region
      style={{
        width: "350px",
        height: "40px",
        backgroundColor: "#2d2d2d",
        borderRadius: "8px",
        display: "flex",
        alignItems: "center",
        padding: "0 12px",
        boxShadow: "0 4px 12px rgba(0, 0, 0, 0.3)",
        userSelect: "none",
        gap: "8px",
        transform: "none",
        zoom: "1",
      }}
    >
      {/* 文字预览 */}
      <div
        style={{
          flex: 1,
          color: "#fff",
          fontSize: "13px",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
        title={selectedText}
      >
        {selectedText.length > 20
          ? selectedText.substring(0, 20) + "..."
          : selectedText}
      </div>

      {/* 复制按钮 */}
      <button
        onClick={handleCopy}
        style={{
          padding: "4px 12px",
          backgroundColor: "#4a9eff",
          color: "#fff",
          border: "none",
          borderRadius: "4px",
          fontSize: "12px",
          cursor: "pointer",
          transition: "background-color 0.2s",
          flexShrink: 0,
        }}
        onMouseEnter={(e) =>
          (e.currentTarget.style.backgroundColor = "#3a8eef")
        }
        onMouseLeave={(e) =>
          (e.currentTarget.style.backgroundColor = "#4a9eff")
        }
      >
        复制
      </button>

      {/* 搜索按钮 */}
      <button
        onClick={handleSearch}
        style={{
          padding: "4px 12px",
          backgroundColor: "#28a745",
          color: "#fff",
          border: "none",
          borderRadius: "4px",
          fontSize: "12px",
          cursor: "pointer",
          transition: "background-color 0.2s",
          flexShrink: 0,
        }}
        onMouseEnter={(e) =>
          (e.currentTarget.style.backgroundColor = "#218838")
        }
        onMouseLeave={(e) =>
          (e.currentTarget.style.backgroundColor = "#28a745")
        }
      >
        搜索
      </button>

      {/* 关闭按钮 */}
      <button
        onClick={handleClose}
        style={{
          width: "20px",
          height: "20px",
          backgroundColor: "transparent",
          color: "#999",
          border: "none",
          fontSize: "14px",
          cursor: "pointer",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          borderRadius: "2px",
          flexShrink: 0,
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = "#ff4444";
          e.currentTarget.style.color = "#fff";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = "transparent";
          e.currentTarget.style.color = "#999";
        }}
      >
        ×
      </button>
    </div>
  );
}

export default FloatingToolbar;
