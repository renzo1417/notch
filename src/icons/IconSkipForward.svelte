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
    d="M13.5 3C13.5 2.44772 13.0523 2 12.5 2C11.9477 2 11.5 2.44772 11.5 3V6.93889L4.4758 2.54877C3.81174 2.13374 2.9516 2.61141 2.9516 3.39384V12.6062C2.9516 13.3886 3.81174 13.8663 4.4758 13.4512L11.5 9.06111V13C11.5 13.5523 11.9477 14 12.5 14C13.0523 14 13.5 13.5523 13.5 13V3Z"
    fill={color}
  />
</svg>
