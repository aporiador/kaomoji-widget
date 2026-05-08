/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import "./styles/app.scss";
import { invoke } from "@tauri-apps/api/core";

if (import.meta.env.DEV) {
  (window as any).invoke = invoke;
}

render(() => <App />, document.getElementById("root") as HTMLElement);
