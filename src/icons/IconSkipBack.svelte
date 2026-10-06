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
    d="M2.5 3C2.5 2.44772 2.94772 2 3.5 2C4.05228 2 4.5 2.44772 4.5 3V6.93889L11.5242 2.54877C12.1883 2.13374 13.0484 2.61141 13.0484 3.39384V12.6062C13.0484 13.3886 12.1883 13.8663 11.5242 13.4512L4.5 9.06111V13C4.5 13.5523 4.05228 14 3.5 14C2.94772 14 2.5 13.5523 2.5 13V3Z"
    fill={color}
  />
</svg>
