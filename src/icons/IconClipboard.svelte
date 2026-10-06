<script lang="ts">
  interface Props {
    size?: number;
    color?: string;
  }

  let { size = 12, color = "currentColor" }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <rect x="3" y="4" width="10" height="10" rx="1.5" stroke={color} stroke-width="1.3" />
  <path d="M6 3C6 2.44772 6.44772 2 7 2H9C9.55228 2 10 2.44772 10 3V4H6V3Z" stroke={color} stroke-width="1.2" />
  <path d="M5.5 8H10.5" stroke={color} stroke-width="1.2" stroke-linecap="round" />
  <path d="M5.5 10.5H9" stroke={color} stroke-width="1.2" stroke-linecap="round" />
</svg>
