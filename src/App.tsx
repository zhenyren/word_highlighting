import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import Logger from "./common/Logger";

function App() {
  const [selectedText, setSelectedText] = useState<string>("");

  useEffect(() => {
    // 监听选中的文字事件
    const unlisten = listen<string>("text_selected", (event) => {
      console.log("选中的文字:", event.payload);
      setSelectedText(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function onLogClick() {
    await Logger.info("word_highlight_assistant", {
      message: "这是一条日志",
      code: 200,
    });
  }

  return (
    <main className="container">
      <div>
        <button onClick={onLogClick}>打印日志</button>
      </div>

      <div
        style={{ marginTop: "20px", padding: "10px", border: "1px solid #ccc" }}
      >
        <h3>选中的文字:</h3>
        {selectedText ? (
          <div
            style={{
              padding: "10px",
              backgroundColor: "#f0f0f0",
              borderRadius: "4px",
              wordBreak: "break-all",
            }}
          >
            {selectedText}
          </div>
        ) : (
          <div style={{ color: "#999" }}>在其他应用中选择文字后松鼠标...</div>
        )}
      </div>
    </main>
  );
}

export default App;
