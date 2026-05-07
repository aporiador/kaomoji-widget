/* @refresh reload */
import { render } from "solid-js/web";
import SettingsApp from "./settings/SettingsApp";
import "./styles/settings.scss";

render(() => <SettingsApp />, document.getElementById("root") as HTMLElement);
