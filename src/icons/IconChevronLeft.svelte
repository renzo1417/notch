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
    d="M10 3.5L5.5 8L10 12.5"
    stroke={color}
    stroke-width="1.4"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>
