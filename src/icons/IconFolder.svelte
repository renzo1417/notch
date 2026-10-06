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
  <path
    d="M2.5 4C2.5 3.45 2.95 3 3.5 3H6.2C6.6 3 6.95 3.2 7.15 3.55L7.85 4.7C8.05 5.05 8.4 5.25 8.8 5.25H12.5C13.05 5.25 13.5 5.7 13.5 6.25V12C13.5 12.55 13.05 13 12.5 13H3.5C2.95 13 2.5 12.55 2.5 12V4Z"
    stroke={color}
    stroke-width="1.3"
    stroke-linejoin="round"
  />
</svg>
