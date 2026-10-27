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
  viewBox="0 0 24 24"
  fill="none"
  stroke={color}
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <circle cx="12" cy="12" r="1" fill={color} />
  <circle cx="19" cy="12" r="1" fill={color} />
  <circle cx="5" cy="12" r="1" fill={color} />
</svg>
