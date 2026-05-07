/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import "./styles/app.scss";

render(() => <App />, document.getElementById("root") as HTMLElement);
