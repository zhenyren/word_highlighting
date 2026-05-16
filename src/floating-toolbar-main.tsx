import React from "react";
import ReactDOM from "react-dom/client";
import "virtual:uno.css";
import "./index.css";
import FloatingToolbar from "./FloatingToolbar";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <FloatingToolbar />
  </React.StrictMode>
);
