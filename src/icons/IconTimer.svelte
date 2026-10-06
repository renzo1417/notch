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
  <circle cx="8" cy="8.5" r="5.5" stroke={color} stroke-width="1.3" />
  <path d="M8 5.5V8.5L10 10.5" stroke={color} stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
  <path d="M6.5 1.5H9.5" stroke={color} stroke-width="1.3" stroke-linecap="round" />
</svg>
