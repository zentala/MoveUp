/**
 * tof_reader.ino
 *
 * Reads distance from VL53L0X or VL53L1X Time-of-Flight sensor
 * via I2C on Seeed XIAO ESP32-C3.
 *
 * XIAO ESP32-C3 Grove I2C pinout:
 *   SDA = GPIO6 (D4)
 *   SCL = GPIO7 (D5)
 *
 * Libraries required (install via Arduino IDE Library Manager):
 *   - "Adafruit_VL53L1X" by Adafruit  (for VL53L1X)
 *   - "Adafruit_VL53L0X" by Adafruit  (for VL53L0X fallback)
 *
 * Serial output: 115200 baud, format: "distance: 1234 mm"
 *
 * Device identification: responds to "PING\n" with "DEVICE: zntl-desk-sensor v1"
 */

#define DEVICE_ID "DEVICE: zntl-desk-sensor v1"

#include <Wire.h>
#include <Adafruit_VL53L1X.h>

#define I2C_SDA 6
#define I2C_SCL 7
#define XSHUT_PIN -1  // not connected, -1 = skip
#define IRQ_PIN   -1

Adafruit_VL53L1X sensor = Adafruit_VL53L1X(XSHUT_PIN, IRQ_PIN);

void setup() {
  Serial.begin(115200);
  while (!Serial) delay(10);

  Serial.println(DEVICE_ID);
  Serial.println("VL53L1X Distance Sensor — XIAO ESP32-C3");

  Wire.begin(I2C_SDA, I2C_SCL);

  if (!sensor.begin(0x29, &Wire)) {
    Serial.print("ERROR: VL53L1X not found. Status: ");
    Serial.println(sensor.vl_status);
    while (1) delay(1000);
  }

  Serial.println("Sensor OK");
  Serial.println("STATUS: OK");

  // Short range: up to ~1.3 m, better accuracy
  // Long range:  up to ~4 m, needs good lighting
  sensor.setTimingBudget(50);  // ms — 20/50/100/200/500

  if (!sensor.startRanging()) {
    Serial.print("ERROR: startRanging failed. Status: ");
    Serial.println(sensor.vl_status);
    while (1) delay(1000);
  }
}

void handleSerial() {
  if (Serial.available()) {
    String cmd = Serial.readStringUntil('\n');
    cmd.trim();
    if (cmd == "PING") {
      Serial.println(DEVICE_ID);
    }
  }
}

void loop() {
  handleSerial();

  if (sensor.dataReady()) {
    int16_t distance = sensor.distance();

    if (distance == -1) {
      Serial.print("ERROR: out of range. Status: ");
      Serial.println(sensor.vl_status);
    } else {
      Serial.print("distance: ");
      Serial.print(distance);
      Serial.println(" mm");
    }

    sensor.clearInterrupt();
  }
  delay(100);
}
