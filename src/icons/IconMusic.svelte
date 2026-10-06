<script lang="ts">
  interface Props {
    size?: number;
    color?: string;
  }

  let { size = 16, color = "currentColor" }: Props = $props();
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
    d="M6 12C6 13.1046 5.10457 14 4 14C2.89543 14 2 13.1046 2 12C2 10.8954 2.89543 10 4 10C4.35418 10 4.68652 10.0924 4.97544 10.2546L5 4.5C5 3.67157 5.67157 3 6.5 3H12.5C13.3284 3 14 3.67157 14 4.5V10.5C14 11.6046 13.1046 12.5 12 12.5C10.8954 12.5 10 11.6046 10 10.5C10 9.39543 10.8954 8.5 12 8.5C12.3542 8.5 12.6865 8.59242 12.9754 8.75462L13 5.5C13 5.22386 12.7761 5 12.5 5H6.5C6.22386 5 6 5.22386 6 5.5V12Z"
    fill={color}
  />
</svg>
