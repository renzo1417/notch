<script lang="ts">
  import { onMount } from "svelte";
  import { launcherStore } from "./launcher-store.svelte";
  import IconApp from "../../icons/IconApp.svelte";
  let { id }: { id: string } = $props();
  let element: HTMLSpanElement;
  let image = $state<string | null>(null);
  let failed = $state(false);
  onMount(() => {
    let cancelled = false;
    const observer = new IntersectionObserver((entries) => {
      if (!entries.some((entry) => entry.isIntersecting)) return;
      observer.disconnect();
      void launcherStore.installedAppIcon(id).then((url) => { if (!cancelled) image = url; });
    });
    observer.observe(element);
    return () => { cancelled = true; observer.disconnect(); };
  });
</script>

<span bind:this={element} class="catalog-icon" aria-hidden="true">
  {#if image && !failed}<img src={image} alt="" onerror={() => { failed = true; }} />{:else}<IconApp size={24} />{/if}
</span>

<style>
  .catalog-icon { width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; flex: none; color: var(--text-secondary); }
  img { width: 28px; height: 28px; object-fit: contain; }
</style>
