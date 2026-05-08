import { createSignal, Show, onMount, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Slider } from "@kobalte/core/slider";
import { Switch } from "@kobalte/core/switch";
import { Select } from "@kobalte/core/select";
import { Settings } from "../lib/settings";
import { DEFAULT_SETTINGS } from "../lib/settings-defaults";
import NotchSettings from "./components/NotchSettings";
import { Separator } from "@kobalte/core/separator";

const FONT_OPTIONS = [
  {
    value:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif',
    label: "System Sans",
  },
  { value: '"Courier New", Courier, monospace', label: "Monospace" },
  { value: '"Times New Roman", Times, serif', label: "Serif" },
  { value: '"Comic Sans MS", "Comic Sans", cursive', label: "Comic Sans" },
  { value: '"Georgia", serif', label: "Georgia" },
  { value: '"Verdana", sans-serif', label: "Verdana" },
];

function getEffectiveTheme(theme: string): "dark" | "light" {
  if (theme === "dark") return "dark";
  if (theme === "light") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

const ThemeIcon = (props: { theme: string }) => {
  return (
    <Show
      when={props.theme === "system"}
      fallback={
        <Show
          when={props.theme === "light"}
          fallback={
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
            </svg>
          }
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="5"></circle>
            <line x1="12" y1="1" x2="12" y2="3"></line>
            <line x1="12" y1="21" x2="12" y2="23"></line>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
            <line x1="1" y1="12" x2="3" y2="12"></line>
            <line x1="21" y1="12" x2="23" y2="12"></line>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
          </svg>
        </Show>
      }
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
        <line x1="8" y1="21" x2="16" y2="21"></line>
        <line x1="12" y1="17" x2="12" y2="21"></line>
      </svg>
    </Show>
  );
};

const THEME_LABELS: Record<string, string> = {
  system: "System",
  light: "Light",
  dark: "Dark",
};

function SettingsApp() {
  const [settings, setSettings] = createSignal<Settings>({
    ...DEFAULT_SETTINGS,
  });
  const [isReady, setIsReady] = createSignal(false);

  onMount(async () => {
    try {
      const stored = await invoke<Settings>("get_settings");
      setSettings({ ...DEFAULT_SETTINGS, ...stored });
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
    setIsReady(true);
  });

  createEffect(() => {
    const effective = getEffectiveTheme(settings().theme);
    document.documentElement.classList.toggle("dark", effective === "dark");
  });

  onMount(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      if (settings().theme === "system") {
        document.documentElement.classList.toggle("dark", mql.matches);
      }
    };
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  });

  const updateSetting = async <K extends keyof Settings>(
    key: K,
    value: Settings[K],
  ) => {
    const next = { ...settings(), [key]: value };
    setSettings(next);
    try {
      await invoke("set_settings", { settings: next });
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  };

  const cycleTheme = () => {
    const order: Array<Settings["theme"]> = ["system", "light", "dark"];
    const current = settings().theme;
    const next = order[(order.indexOf(current) + 1) % order.length];
    updateSetting("theme", next);
  };

  return (
    <div class="p-6 max-w-md mx-auto min-h-screen dark:bg-gray-900 dark:text-gray-100">
      <div class="flex items-center justify-between mb-6">
        <h1 class="text-xl font-semibold">Kaomoji Widget Settings</h1>
        <button
          class="flex items-center gap-2 px-3 py-1.5 text-sm rounded-lg border border-gray-300 hover:border-gray-400 dark:border-gray-600 dark:hover:border-gray-500 transition-colors"
          onClick={cycleTheme}
          title={`Theme: ${THEME_LABELS[settings().theme]}`}
        >
          <ThemeIcon theme={settings().theme} />
          <span>{THEME_LABELS[settings().theme]}</span>
        </button>
      </div>

      <Show
        when={isReady()}
        fallback={
          <div class="text-gray-500 dark:text-gray-400">Loading...</div>
        }
      >
        <div class="space-y-4">
          {/* Font Color */}
          <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Font Color
            </label>
            <div class="flex items-center gap-3">
              <input
                type="color"
                value={settings().font_color}
                onInput={(e) =>
                  updateSetting("font_color", e.currentTarget.value)
                }
                class="w-10 h-10 rounded-lg border border-gray-300 dark:border-gray-600 cursor-pointer overflow-hidden p-0"
              />
              <span class="text-sm text-gray-600 dark:text-gray-400 font-mono">
                {settings().font_color}
              </span>
            </div>
          </div>

          {/* Font Family */}
          <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Font Family
            </label>
            <Select
              options={FONT_OPTIONS}
              optionValue="value"
              optionTextValue="label"
              value={
                FONT_OPTIONS.find((f) => f.value === settings().font_family) ??
                FONT_OPTIONS[0]
              }
              onChange={(opt) => {
                if (opt) updateSetting("font_family", opt.value);
              }}
              itemComponent={(props) => (
                <Select.Item
                  item={props.item}
                  class="px-3 py-2 text-sm cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md data-[highlighted]:bg-gray-100 dark:data-[highlighted]:bg-gray-700"
                >
                  <Select.ItemLabel>
                    {props.item.rawValue.label}
                  </Select.ItemLabel>
                </Select.Item>
              )}
            >
              <Select.Trigger class="flex items-center justify-between w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg hover:border-gray-400 dark:hover:border-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-800 dark:text-gray-100">
                <Select.Value<(typeof FONT_OPTIONS)[0]>>
                  {(state) => state.selectedOption()?.label}
                </Select.Value>
                <Select.Icon class="text-gray-500 dark:text-gray-400">
                  ▼
                </Select.Icon>
              </Select.Trigger>
              <Select.Portal>
                <Select.Content class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-1 z-50">
                  <Select.Listbox class="outline-none" />
                </Select.Content>
              </Select.Portal>
            </Select>
          </div>

          {/* Font Size */}
          <div class="space-y-3">
            <div class="flex justify-between items-center">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Font Size
              </label>
              <span class="text-sm text-gray-600 dark:text-gray-400">
                {settings().font_size}px
              </span>
            </div>
            <Slider
              value={[settings().font_size]}
              onChange={(vals) => updateSetting("font_size", vals[0])}
              minValue={12}
              maxValue={120}
              step={1}
              class="relative flex w-full touch-none select-none flex-col items-center"
            >
              <Slider.Track class="relative h-2 w-full rounded-full bg-gray-200 dark:bg-gray-700">
                <Slider.Fill class="absolute h-full rounded-full bg-blue-500" />
                <Slider.Thumb class="focus-visible:ring-ring absolute -top-1.5 block h-5 w-5 rounded-full border-2 border-blue-500 bg-white dark:bg-gray-800 shadow transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50" />
              </Slider.Track>
            </Slider>
          </div>

          {/* Opacity */}
          <div class="space-y-3">
            <div class="flex justify-between items-center">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Opacity
              </label>
              <span class="text-sm text-gray-600 dark:text-gray-400">
                {Math.round(settings().opacity * 100)}%
              </span>
            </div>
            <Slider
              value={[settings().opacity]}
              onChange={(vals) => updateSetting("opacity", vals[0])}
              minValue={0.1}
              maxValue={1}
              step={0.05}
              class="relative flex w-full touch-none select-none flex-col items-center"
            >
              <Slider.Track class="relative h-2 w-full rounded-full bg-gray-200 dark:bg-gray-700">
                <Slider.Fill class="absolute h-full rounded-full bg-blue-500" />
                <Slider.Thumb class="focus-visible:ring-ring absolute -top-1.5 block h-5 w-5 rounded-full border-2 border-blue-500 bg-white dark:bg-gray-800 shadow transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50" />
              </Slider.Track>
            </Slider>
          </div>

          {/* Text Shadow */}
          <div class="flex items-center justify-between">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
              Text Shadow
            </label>
            <Switch
              checked={settings().text_shadow}
              onChange={(checked) => updateSetting("text_shadow", checked)}
              class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors data-[checked]:bg-blue-500 data-[unchecked]:bg-gray-200 dark:data-[unchecked]:bg-gray-700"
            >
              <Switch.Input class="peer sr-only" />
              <Switch.Control class="inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors data-[checked]:bg-blue-500 data-[unchecked]:bg-gray-200 dark:data-[unchecked]:bg-gray-700">
                <Switch.Thumb class="pointer-events-none block h-5 w-5 rounded-full bg-white dark:bg-gray-200 shadow ring-0 transition-transform data-[checked]:translate-x-5 data-[unchecked]:translate-x-0" />
              </Switch.Control>
            </Switch>
          </div>

          {/* Text Shadow Color */}
          <Show when={settings().text_shadow}>
            <div class="space-y-2">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Text Shadow Color
              </label>
              <div class="flex items-center gap-3">
                <input
                  type="color"
                  value={settings().text_shadow_color}
                  onInput={(e) =>
                    updateSetting("text_shadow_color", e.currentTarget.value)
                  }
                  class="w-10 h-10 rounded-lg border border-gray-300 dark:border-gray-600 cursor-pointer overflow-hidden p-0"
                />
                <span class="text-sm text-gray-600 dark:text-gray-400 font-mono">
                  {settings().text_shadow_color}
                </span>
              </div>
            </div>

            {/* Text Shadow Opacity */}
            <div class="space-y-3">
              <div class="flex justify-between items-center">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Text Shadow Opacity
                </label>
                <input
                  type="number"
                  min={0}
                  max={1}
                  step={0.01}
                  value={settings().text_shadow_opacity}
                  onInput={(e) => {
                    const val = parseFloat(e.currentTarget.value);
                    if (!isNaN(val))
                      updateSetting(
                        "text_shadow_opacity",
                        Math.min(1, Math.max(0, val)),
                      );
                  }}
                  class="w-16 text-sm text-right text-gray-600 dark:text-gray-400 border border-gray-300 dark:border-gray-600 rounded-lg px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
                />
              </div>
              <Slider
                value={[settings().text_shadow_opacity]}
                onChange={(vals) =>
                  updateSetting("text_shadow_opacity", vals[0])
                }
                minValue={0}
                maxValue={1}
                step={0.01}
                class="relative flex w-full touch-none select-none flex-col items-center"
              >
                <Slider.Track class="relative h-2 w-full rounded-full bg-gray-200 dark:bg-gray-700">
                  <Slider.Fill class="absolute h-full rounded-full bg-blue-500" />
                  <Slider.Thumb class="focus-visible:ring-ring absolute -top-1.5 block h-5 w-5 rounded-full border-2 border-blue-500 bg-white dark:bg-gray-800 shadow transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50" />
                </Slider.Track>
              </Slider>
            </div>
          </Show>

          {/* Background Color */}
          <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Background Color
            </label>
            <div class="flex items-center gap-3">
              <input
                type="color"
                value={settings().background_color}
                onInput={(e) =>
                  updateSetting("background_color", e.currentTarget.value)
                }
                class="w-10 h-10 rounded-lg border border-gray-300 dark:border-gray-600 cursor-pointer overflow-hidden p-0"
              />
              <span class="text-sm text-gray-600 dark:text-gray-400 font-mono">
                {settings().background_color}
              </span>
            </div>
          </div>

          {/* Background Opacity */}
          <div class="space-y-3">
            <div class="flex justify-between items-center">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Background Opacity
              </label>
              <input
                type="number"
                min={0}
                max={1}
                step={0.01}
                value={settings().background_opacity}
                onInput={(e) => {
                  const val = parseFloat(e.currentTarget.value);
                  if (!isNaN(val))
                    updateSetting(
                      "background_opacity",
                      Math.min(1, Math.max(0, val)),
                    );
                }}
                class="w-16 text-sm text-right text-gray-600 dark:text-gray-400 border border-gray-300 dark:border-gray-600 rounded-lg px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
              />
            </div>
            <Slider
              value={[settings().background_opacity]}
              onChange={(vals) => updateSetting("background_opacity", vals[0])}
              minValue={0}
              maxValue={1}
              step={0.01}
              class="relative flex w-full touch-none select-none flex-col items-center"
            >
              <Slider.Track class="relative h-2 w-full rounded-full bg-gray-200 dark:bg-gray-700">
                <Slider.Fill class="absolute h-full rounded-full bg-blue-500" />
                <Slider.Thumb class="focus-visible:ring-ring absolute -top-1.5 block h-5 w-5 rounded-full border-2 border-blue-500 bg-white dark:bg-gray-800 shadow transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50" />
              </Slider.Track>
            </Slider>
          </div>

          {/* Notch Mode */}
          <Separator />
          <NotchSettings settings={settings()} onUpdate={updateSetting} />
        </div>
      </Show>
    </div>
  );
}

export default SettingsApp;
