import {
  createSignal,
  Switch,
  Match,
  onMount,
  onCleanup,
  createEffect,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { onDisplayUpdate, onSettingsUpdate, DisplayPayload } from "./lib/ipc";
import { Settings } from "./lib/settings";
import { DEFAULT_SETTINGS } from "./lib/settings-defaults";
import { prepareWithSegments, measureNaturalWidth } from "@chenglou/pretext";

function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function getTextWidth(text: string, fontFamily: string): number {
  const prepared = prepareWithSegments(text, `100px ${fontFamily}`);
  return measureNaturalWidth(prepared);
}

function App() {
  const [display, setDisplay] = createSignal<DisplayPayload>({
    kind: "kaomoji",
    text: "(´• ω •`)",
  });

  const [settings, setSettings] = createSignal<Settings>({
    ...DEFAULT_SETTINGS,
  });

  let kaomojiRef: HTMLSpanElement | undefined;

  const fitText = (text: string) => {
    if (!kaomojiRef) return;
    const container = kaomojiRef.parentElement!;
    const maxW = container.clientWidth;
    const maxH = container.clientHeight;

    // Width-based font size (existing behaviour — pretext handles complex Unicode well).
    const textWidth = getTextWidth(text, settings().font_family);
    const fSizeFromWidth = settings().font_size * (maxW / textWidth);

    // Height-based maximum font size.
    //
    // The browser centres the *line box* (height = font-size) in the flex container,
    // then `position:relative; top:-4px` shifts it upward visually. Characters whose
    // ink exceeds the line box — like ‿ (U+203F UNDERTIE) which sits well below the
    // baseline — can therefore overflow the container and be clipped.
    //
    // Layout model (line-height:1, flex centre, top:-4px):
    //   lineCenter = maxH/2 − CSS_TOP_OFFSET
    //   lineBox.top = lineCenter − F/2
    //   baseline    = lineBox.top + fontBoundingBoxAscent(F)
    //               = lineCenter + F·(fba − refSize/2) / refSize
    //   glyphBottom = baseline + actualBoundingBoxDescent(F)
    //               = lineCenter + F·(fba + abd − refSize/2) / refSize
    //
    // Solving glyphBottom ≤ maxH for F:
    //   F ≤ (maxH − lineCenter)·refSize / (fba + abd − refSize/2)
    //
    // Similarly for the top (characters like ◕ can extend above the em-box):
    //   glyphTop = lineCenter + F·(fba − aba − refSize/2) / refSize ≥ 0
    //   ⟹ F ≤ lineCenter·refSize / (aba − fba + refSize/2)  [when aba > fba − refSize/2]
    const refSize = 100;
    // Read the actual CSS top offset so this stays correct if the CSS changes.
    // `top: -4px` → computedTop = "-4px" → topOffset = -4 → lineCenter shifts up by 4.
    const topOffset = parseFloat(getComputedStyle(kaomojiRef).top) || 0;
    const lineCenter = maxH / 2 + topOffset; // topOffset is negative, so this shifts down

    const ctx = document.createElement("canvas").getContext("2d")!;
    ctx.font = `600 ${refSize}px ${settings().font_family}`;
    const m = ctx.measureText(text);
    const fba = m.fontBoundingBoxAscent; // font's built-in ascent metric
    const aba = m.actualBoundingBoxAscent; // actual ink ascent for this text
    const abd = m.actualBoundingBoxDescent; // actual ink descent for this text

    const bottomDenom = fba + abd - refSize / 2;
    const fSizeFromBottom =
      bottomDenom > 0
        ? ((maxH - lineCenter) * refSize) / bottomDenom
        : Infinity;

    const topDenom = aba - fba + refSize / 2; // positive when top might clip
    const fSizeFromTop =
      topDenom > 0 && aba > fba - refSize / 2
        ? (lineCenter * refSize) / topDenom
        : Infinity;

    const newSize = Math.max(
      6.4,
      Math.min(fSizeFromWidth, fSizeFromBottom, fSizeFromTop),
    );
    kaomojiRef.style.fontSize = `${newSize}px`;
  };

  createEffect(() => {
    const d = display();
    if (d.kind === "kaomoji" && d.text) {
      fitText(d.text);
    }
  });

  onMount(() => {
    let unlistenDisplay: (() => void) | undefined;
    let unlistenSettings: (() => void) | undefined;

    // Load initial settings
    invoke<Settings>("get_settings")
      .then((s) => setSettings({ ...DEFAULT_SETTINGS, ...s }))
      .catch((e) => console.error("Failed to load settings:", e));

    onDisplayUpdate((payload) => {
      setDisplay(payload);
    }).then((fn) => {
      unlistenDisplay = fn;
    });

    onSettingsUpdate((payload) => {
      setSettings(payload);
    }).then((fn) => {
      unlistenSettings = fn;
    });

    onCleanup(() => {
      if (unlistenDisplay) unlistenDisplay();
      if (unlistenSettings) unlistenSettings();
    });
  });

  const kaomojiStyle = () => {
    const s = settings();
    return {
      color: s.font_color,
      "font-family": s.font_family,
      opacity: s.opacity,
      "text-shadow": s.text_shadow
        ? `0 2px 4px ${hexToRgba(s.text_shadow_color, s.text_shadow_opacity)}`
        : "none",
    };
  };

  return (
    <div
      class="widget"
      data-tauri-drag-region
      style={{
        "background-color": hexToRgba(
          settings().background_color,
          settings().background_opacity,
        ),
      }}
    >
      <Switch fallback={null}>
        <Match when={display().kind === "kaomoji"}>
          <span class="kaomoji" ref={kaomojiRef} style={kaomojiStyle()}>
            {(display() as Extract<DisplayPayload, { kind: "kaomoji" }>).text}
          </span>
        </Match>
        <Match when={display().kind === "image"}>
          <img
            class="display-image"
            src={(display() as Extract<DisplayPayload, { kind: "image" }>).data}
            alt="widget display"
          />
        </Match>
        <Match when={display().kind === "empty"}>
          <span class="kaomoji" style={kaomojiStyle()}></span>
        </Match>
      </Switch>
    </div>
  );
}

export default App;
