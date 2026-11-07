<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import CompactNotch from "./CompactNotch.svelte";
  import ExpandedNotch from "./ExpandedNotch.svelte";
  import { captureStore } from "../modules/capture/capture-store.svelte";
  import SettingsView from "../settings/SettingsView.svelte";
  import { backState, SHELL_STATE_DIMENSIONS, shellActions, type NotchShellState } from "../app/state/shell-state";
  import { ShellTransition } from "./shell-transition";
  import { settingsStore } from "../settings/settings-store.svelte";
  import { launcherStore } from "../modules/launcher/launcher-store.svelte";
  import { timerStore } from "../modules/timer/timer-store.svelte";
  import { clipboardStore } from "../modules/clipboard/clipboard-store.svelte";
  import { downloadsStore } from "../modules/downloads/downloads-store.svelte";
  import { mediaStore } from "../modules/music/media-store.svelte";
  import { codingStore } from "../modules/coding/coding-store.svelte";
  import { batteryStore } from "../modules/battery/battery-store.svelte";

  let shellState = $state<NotchShellState>("compact");
  let shell: HTMLDivElement;
  let resizeError = $state("");
  let pointerInside = false;
  let keyboardActive = false;
  let autoCollapse: ReturnType<typeof setTimeout> | null = null;
  let hasOpened = $state(false);
  let hasSettings = $state(false);
  let pickerOrigin: NotchShellState = "apps";
  let pickerReplaceId = $state<string | undefined>();
  const savedFocus = new Map<NotchShellState, HTMLElement>();
  let unsubscribePicker: (() => void) | undefined;
  let unsubscribeCollapse: (() => void) | undefined;
  let unlistenSettings: UnlistenFn | undefined;
  let initialResize: Promise<void> = Promise.resolve();
  let disposed = false;
  const geometry = $derived(SHELL_STATE_DIMENSIONS[shellState]);
  const native = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

  async function resize(state: NotchShellState) {
    if (native) await invoke("set_notch_state", { notchState: state });
  }

  function clearAutoCollapse() {
    if (autoCollapse) clearTimeout(autoCollapse);
    autoCollapse = null;
  }

  function startAutoCollapse() {
    clearAutoCollapse();
    const delay = settingsStore.state.general.autoCollapseDelayMs;
    if (!captureStore.busy && shellState === "expanded" && !pointerInside && !(keyboardActive && shell?.contains(document.activeElement)) && delay > 0) {
      autoCollapse = setTimeout(() => { void navigate("compact"); }, delay);
    }
  }

  function closeSurface(state: NotchShellState) {
    if (state === "apps") launcherStore.closeLauncherSurface();
    if (state === "focus") timerStore.closeTimerSurface();
    if (state === "clipboard") clipboardStore.closeClipboardSurface();
    if (state === "downloads") downloadsStore.closeDownloadSurface();
    if (state === "battery") batteryStore.closeBatterySurface();
  }

  function openSurface(state: NotchShellState) {
    if (state === "apps") launcherStore.openLauncherSurface();
    if (state === "focus") timerStore.openTimerSurface();
    if (state === "clipboard") clipboardStore.openClipboardSurface();
    if (state === "downloads") downloadsStore.openDownloadSurface();
    if (state === "battery") batteryStore.openBatterySurface();
  }

  function show(state: NotchShellState) {
    const previous = shellState;
    closeSurface(previous);
    shellState = state;
    if (state !== "compact") hasOpened = true;
    if (state === "settings") hasSettings = true;
    openSurface(state);
    void tick().then(() => {
      if (disposed || shellState !== state) return;
      const parentNav = previous === "apps" ? "apps" : previous === "focus" ? "focus" : previous === "capture" ? "capture" : "more";
      const restored = state === "expanded" && previous !== "compact"
        ? shell.querySelector<HTMLElement>(`[data-focus="${parentNav}"]`) ?? savedFocus.get(state)
        : savedFocus.get(state);
      if (restored?.isConnected && restored.getClientRects().length && !restored.hasAttribute("disabled")) {
        restored.focus({ preventScroll: true });
      } else {
        const selector = state === "compact" ? '[data-focus="compact"]' :
          state === "expanded" ? '[data-focus="collapse"]' :
          state === "settings" ? '[data-focus="settings"]' :
          state === "appPicker" ? '[data-autofocus]' : '[data-panel-focus]';
        shell.querySelector<HTMLElement>(selector)?.focus({ preventScroll: true });
      }
      startAutoCollapse();
    });
  }

  const transition = new ShellTransition({
    resize: async (state) => { await initialResize; await resize(state); },
    show,
    duration: () => parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--motion-shell")) || 0,
  });

  async function navigate(state: NotchShellState) {
    if (captureStore.busy) return;
    const active = document.activeElement;
    if (active instanceof HTMLElement && shell?.contains(active)) savedFocus.set(shellState, active);
    clearAutoCollapse();
    resizeError = "";
    try { await transition.request(state); }
    catch { resizeError = "Couldn’t resize the notch. Try again."; }
  }

  function back() { if (captureStore.busy) void captureStore.cancel(); else void navigate(backState(shellState, pickerOrigin)); }

  function handleKeyDown(event: KeyboardEvent) {
    keyboardActive = true;
    if (event.key === "Escape" && shellState !== "compact") {
      event.preventDefault();
      event.stopPropagation();
      back();
    }
  }

  onMount(() => {
    initialResize = resize("compact").catch(() => { resizeError = "Couldn’t position the notch. Try opening it again."; });
    void settingsStore.init();
    void mediaStore.init();
    void codingStore.init();
    void clipboardStore.init();
    void batteryStore.init();
    void downloadsStore.init();
    unsubscribePicker = shellActions.onAppPicker((replaceId) => {
      if (captureStore.busy) return;
      pickerOrigin = shellState === "settings" ? "settings" : "apps";
      pickerReplaceId = replaceId;
      void navigate("appPicker");
    });
    unsubscribeCollapse = shellActions.onCollapse(() => { void navigate("compact"); });
    const motionPreference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotion = () => settingsStore.applyLiveTokens(settingsStore.state);
    motionPreference.addEventListener("change", updateMotion);
    if (native) void listen("open-settings", () => { void navigate("settings"); }).then((unlisten) => {
      if (disposed) unlisten(); else unlistenSettings = unlisten;
    });
    return () => motionPreference.removeEventListener("change", updateMotion);
  });

  onDestroy(() => {
    disposed = true;
    transition.destroy();
    clearAutoCollapse();
    unsubscribeCollapse?.();
    unsubscribePicker?.();
    void captureStore.cancel();
    unlistenSettings?.();
    closeSurface(shellState);
    mediaStore.destroy(); codingStore.destroy(); clipboardStore.destroy();
    batteryStore.destroy(); downloadsStore.destroy(); timerStore.destroy(); settingsStore.destroy();
  });
</script>

<svelte:window onkeydown={handleKeyDown} onpointerdown={() => { keyboardActive = false; }} />

<div class="notch-wrapper">
  <div bind:this={shell} class="notch-shell" class:compact={shellState === "compact"}
    style:--shell-width={`${geometry.width}px`} style:--shell-height={`${geometry.height}px`}
    role="region" aria-label="Windows Notch" tabindex="-1"
    onmouseenter={() => { pointerInside = true; clearAutoCollapse(); }}
    onmouseleave={() => { pointerInside = false; startAutoCollapse(); }}
    onfocusin={clearAutoCollapse} onfocusout={() => { setTimeout(startAutoCollapse, 0); }}>
    {#if shellState === "compact"}
      <CompactNotch onopen={() => { void navigate("expanded"); }} />
    {/if}
    {#if hasSettings}
      <div class="shell-surface" hidden={shellState !== "settings"}><SettingsView onclose={back} oncollapse={() => { void navigate("compact"); }} /></div>
    {/if}
    {#if hasOpened}
      <div class="shell-surface" hidden={shellState === "settings" || shellState === "compact"}>
        <ExpandedNotch {shellState} {pickerReplaceId} onStateChange={(state) => { void navigate(state); }} onback={back} oncollapse={() => { void navigate("compact"); }} />
      </div>
    {/if}
    {#if resizeError}<div class="resize-error" role="alert">{resizeError}</div>{/if}
  </div>
</div>

<style>
  .notch-wrapper { display: flex; justify-content: center; align-items: flex-start; width: 100%; height: 100%; padding: 0 16px 16px; pointer-events: none; }
  .notch-shell { position: relative; pointer-events: auto; width: var(--shell-width); height: var(--shell-height); max-width: 100%; max-height: 100%; flex: none; overflow: hidden; outline: none; background: var(--surface-root); border: 1px solid var(--border-subtle); border-top: none; border-radius: 0 0 var(--radius-shell) var(--radius-shell); box-shadow: var(--shadow-soft); transition: width var(--motion-shell) var(--ease-natural), height var(--motion-shell) var(--ease-natural); }
  .shell-surface { height: 100%; min-height: 0; }
  [hidden] { display: none !important; }
  .resize-error { position: absolute; inset: auto 8px 4px; background: var(--surface-raised); color: var(--status-error); font-size: 12px; padding: 4px 8px; border-radius: 6px; }
</style>
