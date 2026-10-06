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
  <circle cx="8" cy="8" r="6" stroke={color} stroke-width="1.3" />
  <path d="M8 5V8.5" stroke={color} stroke-width="1.4" stroke-linecap="round" />
  <circle cx="8" cy="11" r="0.75" fill={color} />
</svg>
