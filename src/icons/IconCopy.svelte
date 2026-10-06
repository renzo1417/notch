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
  <rect x="5" y="5" width="8" height="8" rx="1.5" stroke={color} stroke-width="1.2" />
  <path d="M3 11V3.5C3 2.67157 3.67157 2 4.5 2H11" stroke={color} stroke-width="1.2" stroke-linecap="round" />
</svg>
