/**
 * DeskDeviceEmulator.ts
 *
 * Emulates the zntl-desk-sensor serial device for testing.
 * Can be used with Rust's `serialport` crate via Windows named pipes
 * or via TypeScript integration tests to inject readings directly.
 */

export interface EmulatorOptions {
  pipeName?: string;
  verbose?: boolean;
}

export class DeskDeviceEmulator {
  static readonly DEVICE_ID = "DEVICE: zntl-desk-sensor v1";
  private verbose: boolean = false;

  constructor(options: EmulatorOptions = {}) {
    this.verbose = options.verbose ?? false;
  }

  /**
   * Response to PING — device identification string.
   */
  static formatPong(): string {
    return DeskDeviceEmulator.DEVICE_ID + "\n";
  }

  /**
   * Format a distance reading in millimeters.
   * Example: "distance: 750 mm\n"
   */
  static formatDistance(mm: number): string {
    return `distance: ${mm} mm\n`;
  }

  /**
   * Format an error message.
   * Example: "ERROR: sensor timeout\n"
   */
  static formatError(msg: string): string {
    return `ERROR: ${msg}\n`;
  }

  /**
   * Send a series of identical distance readings (for debouncing tests).
   * Returns the formatted strings that would be sent.
   */
  static generateReadings(mm: number, count: number = 5): string[] {
    return Array(count).fill(DeskDeviceEmulator.formatDistance(mm));
  }

  private log(msg: string): void {
    if (this.verbose) {
      console.log(`[DeskDeviceEmulator] ${msg}`);
    }
  }
}

export default DeskDeviceEmulator;
