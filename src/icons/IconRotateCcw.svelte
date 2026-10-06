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
    d="M2.5 6.5C3.1 4.2 5.1 2.5 7.5 2.5C10.5376 2.5 13 4.96243 13 8C13 11.0376 10.5376 13.5 7.5 13.5C5.2 13.5 3.2 12.1 2.3 10.1"
    stroke={color}
    stroke-width="1.3"
    stroke-linecap="round"
  />
  <path
    d="M2 3.5V6.5H5"
    stroke={color}
    stroke-width="1.3"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>
