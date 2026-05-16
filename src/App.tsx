import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  async function onShowWordHighlightAssistantWindowClick() {
    await invoke("show_word_highlight_assistant_window");
  }
  async function onCloseWordHighlightAssistantWindowClick() {
    await invoke("close_word_highlight_assistant_window");
  }

  async function onGetClipboardClick() {
    try {
      const text = await navigator.clipboard.readText();
      await invoke("show_text_in_window", { text });
    } catch (err) {
      console.error("Failed to read clipboard:", err);
    }
  }

  return (
    <main className="container">
      <p>{greetMsg}</p>
      <div>
        <button onClick={onShowWordHighlightAssistantWindowClick}>
          打开组件
        </button>
      </div>
      <div>
        <button onClick={onCloseWordHighlightAssistantWindowClick}>
          关闭组件
        </button>
      </div>
      <div>
        <button onClick={onGetClipboardClick}>
          获取剪贴板内容并显示
        </button>
      </div>
    </main>
  );
}

export default App;
