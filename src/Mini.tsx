import React from "react";
import "./Mini.css";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

async function startDrag() {
  await appWindow.startDragging();
}

async function closeWindow() {
  await appWindow.close();
}

function Mini() {
  return (
    <div className="size-full bg-red rounded-12px overflow-hidden flex items-center">
      <div
        className="w-30px h-100% bg-amber cursor-pointer"
        onMouseDown={startDrag}
      ></div>
      <div
        className="absolute top-0 right-0 cursor-pointer"
        onClick={closeWindow}
      ></div>
    </div>
  );
}

export default Mini;
