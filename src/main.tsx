import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { applyPreset } from "./lib/theme";
import "./index.css";

applyPreset("violet");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
