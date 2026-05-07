import { listen } from "@tauri-apps/api/event";
import { Settings } from "./settings";

export type DisplayPayload =
  | { kind: "kaomoji"; text: string }
  | { kind: "image"; data: string; mime: string }
  | { kind: "empty" };

export function onDisplayUpdate(
  callback: (payload: DisplayPayload) => void
): Promise<() => void> {
  return listen<DisplayPayload>("display-update", (event) => {
    callback(event.payload);
  });
}

export function onSettingsUpdate(
  callback: (payload: Settings) => void
): Promise<() => void> {
  return listen<Settings>("settings-update", (event) => {
    callback(event.payload);
  });
}
