<script lang="ts">
  interface Props {
    size?: number;
    color?: string;
  }

  let { size = 14, color = "currentColor" }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <rect
    x="2.5"
    y="2.5"
    width="11"
    height="11"
    rx="2.5"
    stroke={color}
    stroke-width="1.4"
  />
  <path
    d="M5.5 6.5H10.5M5.5 9.5H8.5"
    stroke={color}
    stroke-width="1.4"
    stroke-linecap="round"
  />
</svg>
