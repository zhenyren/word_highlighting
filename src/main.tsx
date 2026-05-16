import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import "virtual:uno.css";
import "./index.css";
import App from "./App";
// import { getCurrentWindow } from "@tauri-apps/api/window";

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

// 启动全局监听

function Root() {
  useEffect(() => {
    // 监听 全局鼠标移动
    invoke("start_global_hook");
    listen("global_mouse_move", (e) => {
      const [x, y] = e.payload as number[];
      // console.log("全局鼠标：", x, y);
    });

    // 监听 全局鼠标按下
    listen("global_mouse_down", (e) => {
      // console.log("全局按下：", e.payload);
    });

    // 监听 全局鼠标松开
    listen("global_mouse_up", (e) => {
      // console.log("全局松开：", e.payload);
    });

    // 监听 全局滚轮
    listen("global_wheel", (e) => {
      // console.log("滚轮：", e.payload);
    });
  }, []);

  return (
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <Root />,
);
