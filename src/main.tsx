import React from "react";
import ReactDOM from "react-dom/client";
import "virtual:uno.css";
import "./index.css";
import App from "./App";
import Mini from "./Mini";
import { getCurrentWindow } from "@tauri-apps/api/window";

async function init() {
  const window = getCurrentWindow();
  const label = window.label;

  const Component =
    label === "word_highlight_assistant" || label === "mini" ? Mini : App;

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <Component />
    </React.StrictMode>,
  );
}

init();
