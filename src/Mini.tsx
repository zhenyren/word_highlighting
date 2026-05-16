import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import "./Mini.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import {
  faGripLinesVertical,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import { useEffect, useState } from "react";

function Mini() {
  const appWindow = getCurrentWindow();
  const [selectedText, setSelectedText] = useState("选择文本...");

  useEffect(() => {
    const unsubscribe = listen("selection-changed", (event) => {
      setSelectedText(event.payload as string);
    });

    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, []);

  async function closeWindow() {
    await appWindow.hide();
  }

  async function startDrag(e: React.MouseEvent) {
    if (e.button === 0) {
      await appWindow.startDragging();
    }
  }

  return (
    <div
      className="size-full bg-white absolute rounded-12px overflow-hidden flex items-center border border-black drag-region"
      data-tauri-drag-region
      onMouseDown={startDrag}
    >
      <div className="w-30px h-100% items-center justify-center flex">
        <FontAwesomeIcon icon={faGripLinesVertical} />
      </div>
      <div className="flex-1 px-2 truncate text-sm">{selectedText}</div>
      <div
        className="absolute top-0 right-0 cursor-pointer w-30px h-full flex items-center justify-center no-drag"
        onClick={closeWindow}
        data-tauri-drag-region="false"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <FontAwesomeIcon icon={faXmark} size="sm" />
      </div>
    </div>
  );
}

export default Mini;
