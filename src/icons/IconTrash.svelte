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
    d="M3 4.5H13M6 4.5V3C6 2.45 6.45 2 7 2H9C9.55 2 10 2.45 10 3V4.5M12 4.5V13C12 13.55 11.55 14 11 14H5C4.45 14 4 13.55 4 13V4.5"
    stroke={color}
    stroke-width="1.3"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
  <path
    d="M6.5 7.5V11M9.5 7.5V11"
    stroke={color}
    stroke-width="1.3"
    stroke-linecap="round"
  />
</svg>
