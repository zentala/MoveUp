/** A valid distance reading emitted by the VL53L1X sensor. */
export interface DistanceReading {
  mm: number;
  cm: number;
  timestamp: string; // ISO string from Rust
}

/** Sensor or serial error event payload. */
export interface SensorError {
  message: string;
  timestamp: string;
}

/** Serial port info returned by list_ports command. */
export interface PortInfo {
  name: string;
  description: string | null;
}
