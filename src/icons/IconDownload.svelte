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
    d="M8 2.5V10.5M8 10.5L5 7.5M8 10.5L11 7.5"
    stroke={color}
    stroke-width="1.3"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
  <path
    d="M2.5 11.5V12.5C2.5 13.05 2.95 13.5 3.5 13.5H12.5C13.05 13.5 13.5 13.05 13.5 12.5V11.5"
    stroke={color}
    stroke-width="1.3"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>
