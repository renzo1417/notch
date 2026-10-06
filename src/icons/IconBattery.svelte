<script lang="ts">
  interface Props {
    size?: number;
    color?: string;
    charging?: boolean;
    level?: number; // 0..100
  }

  let { size = 12, color = "currentColor", charging = false, level = 100 }: Props = $props();

  const fillWidth = $derived(Math.max(1, Math.min(8, Math.round((level / 100) * 8))));
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <!-- Outer chassis -->
  <rect x="2" y="4.5" width="10.5" height="7" rx="1.5" stroke={color} stroke-width="1.2" />
  <!-- Terminal tip -->
  <path d="M13.5 6.5V9.5" stroke={color} stroke-width="1.2" stroke-linecap="round" />
  
  {#if charging}
    <!-- Charging bolt -->
    <path d="M7.5 5.5L5.8 8.5H8L7 10.5L9.2 7.5H7.2L7.5 5.5Z" fill={color} />
  {:else}
    <!-- Level bar -->
    <rect x="3.25" y="5.75" width={fillWidth} height="4.5" rx="0.75" fill={color} />
  {/if}
</svg>
