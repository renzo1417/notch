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
    d="M4.5 3.32832C4.5 2.54589 5.36014 2.06822 6.0242 2.48325L13.4989 7.15494C14.1333 7.55146 14.1333 8.44854 13.4989 8.84506L6.0242 13.5168C5.36014 13.9318 4.5 13.4541 4.5 12.6717V3.32832Z"
    fill={color}
  />
</svg>
