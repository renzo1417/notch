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
  <rect x="3.5" y="2.5" width="3" height="11" rx="1" fill={color} />
  <rect x="9.5" y="2.5" width="3" height="11" rx="1" fill={color} />
</svg>
