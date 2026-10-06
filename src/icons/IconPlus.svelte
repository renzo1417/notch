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
    d="M8 3V13M3 8H13"
    stroke={color}
    stroke-width="1.6"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>
