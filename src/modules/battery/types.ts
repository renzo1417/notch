/**
 * Battery and power state model.
 * Defined in PRODUCT_SPEC.md section 18.
 */
export interface BatteryState {
  percentage: number;
  isCharging: boolean;
  isAcConnected: boolean;
  remainingSeconds?: number;
}
