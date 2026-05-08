import { createSignal, Show, onMount } from "solid-js";
import { Switch } from "@kobalte/core/switch";
import { invoke } from "@tauri-apps/api/core";
import { Settings } from "../../lib/settings";

interface MonitorInfo {
  name: string;
  y: number;
  x: number;
  width: number;
  height: number;
  is_primary: number;
  insets: Insets;
}

interface Insets {
  top: number;
  left: number;
  bottom: number;
  right: number;
}

interface NotchSettingsProps {
  settings: Settings;
  onUpdate: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
}

function computeLayout(
  monitors: MonitorInfo[],
  containerWidth: number,
  containerHeight: number,
) {
  if (monitors.length === 0) return [];

  const minX = Math.min(...monitors.map((m) => m.x));
  const minY = Math.min(...monitors.map((m) => m.y));
  const maxX = Math.max(...monitors.map((m) => m.x + m.width));
  const maxY = Math.max(...monitors.map((m) => m.y + m.height));

  const totalW = maxX - minX;
  const totalH = maxY - minY;

  const padding = 16;
  const availW = Math.max(containerWidth - padding * 2, 1);
  const availH = Math.max(containerHeight - padding * 2, 1);

  const scale = Math.min(availW / totalW, availH / totalH);

  const offsetX = (containerWidth - totalW * scale) / 2 - minX * scale;
  const offsetY = (containerHeight - totalH * scale) / 2 - minY * scale;

  return monitors.map((m) => ({
    ...m,
    left: m.x * scale + offsetX,
    top: m.y * scale + offsetY,
    renderWidth: m.width * scale,
    renderHeight: m.height * scale,
  }));
}

export default function NotchSettings(props: NotchSettingsProps) {
  const [monitors, setMonitors] = createSignal<MonitorInfo[]>([]);
  const [containerRef, setContainerRef] = createSignal<HTMLDivElement | null>(
    null,
  );

  onMount(async () => {
    try {
      const ms = await invoke<MonitorInfo[]>("get_monitors");
      setMonitors(ms);
    } catch (e) {
      console.error("Failed to get monitors:", e);
    }
  });

  const layout = () => {
    const el = containerRef();
    if (!el || monitors().length === 0) return [];
    return computeLayout(monitors(), el.clientWidth, el.clientHeight);
  };

  const selectedName = () => props.settings.notch_monitor;

  return (
    <div class="space-y-1">
      <label class="text-m font-medium text-gray-700 dark:text-gray-400">
        Notch Mode
      </label>
      {/* Notch Mode Toggle */}
      <div class="flex items-center justify-between">
        <div class="space-y-1">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
            Enable Notch
          </label>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            Snap to the top-center camera notch area.
          </p>
        </div>
        <Switch
          checked={props.settings.notch_mode ?? false}
          onChange={(checked) => props.onUpdate("notch_mode", checked)}
          class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors data-[checked]:bg-blue-500 data-[unchecked]:bg-gray-200 dark:data-[unchecked]:bg-gray-700"
        >
          <Switch.Input class="peer sr-only" />
          <Switch.Control class="inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors data-[checked]:bg-blue-500 data-[unchecked]:bg-gray-200 dark:data-[unchecked]:bg-gray-700">
            <Switch.Thumb class="pointer-events-none block h-5 w-5 rounded-full bg-white dark:bg-gray-200 shadow ring-0 transition-transform data-[checked]:translate-x-5 data-[unchecked]:translate-x-0" />
          </Switch.Control>
        </Switch>
      </div>

      {/* Monitor Selector */}
      <div class="space-y-2">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Notch Monitor
        </label>
        <p class="text-xs text-gray-500 dark:text-gray-400">
          Choose which monitor to snap to.
        </p>
        <div
          ref={setContainerRef}
          class="relative h-40 w-full bg-gray-100 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden"
        >
          {layout().map((m) => {
            const isSelected = selectedName() === m.name;
            return (
              <button
                type="button"
                class={`absolute rounded-md border-2 transition-all flex flex-col items-center justify-center p-1 text-center cursor-pointer
                    ${
                      isSelected
                        ? "border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300"
                        : "border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:border-gray-400 dark:hover:border-gray-500"
                    }`}
                style={{
                  left: `${m.left}px`,
                  top: `${m.top}px`,
                  width: `${m.renderWidth}px`,
                  height: `${m.renderHeight}px`,
                }}
                onClick={() => props.onUpdate("notch_monitor", m.name)}
                title={m.name}
              >
                <span class="text-[10px] leading-tight font-medium truncate w-full">
                  {m.name}
                </span>
                <Show when={m.is_primary}>
                  <span class="text-[8px] text-gray-400 dark:text-gray-500 mt-0.5">
                    Primary
                  </span>
                </Show>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
