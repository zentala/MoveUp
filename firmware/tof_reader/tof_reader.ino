/**
 * tof_reader.ino
 *
 * Reads distance from VL53L0X Time-of-Flight sensor
 * via I2C on Seeed XIAO ESP32-C3.
 *
 * XIAO ESP32-C3 Grove I2C pinout (verified by I2C scan):
 *   SDA = GPIO6 (D4)
 *   SCL = GPIO7 (D5)
 *
 * Libraries required (install via Arduino IDE Library Manager):
 *   - "Adafruit_VL53L0X" by Adafruit
 *
 * Serial output: 115200 baud, format: "distance: 1234 mm"
 *
 * Device identification: sends DEVICE_ID on boot and responds to "PING\n"
 */

#define DEVICE_ID "DEVICE: zntl-desk-sensor v1"

#include <Wire.h>
#include <Adafruit_VL53L0X.h>

#define I2C_SDA 6
#define I2C_SCL 7

Adafruit_VL53L0X sensor;

void setup() {
  Serial.begin(115200);
  while (!Serial) delay(10);

  Serial.println(DEVICE_ID);
  Serial.println("VL53L0X Distance Sensor — XIAO ESP32-C3");

  Wire.begin(I2C_SDA, I2C_SCL);

  if (!sensor.begin(VL53L0X_I2C_ADDR, false, &Wire)) {
    Serial.println("ERROR: VL53L0X not found");
    while (1) delay(1000);
  }

  Serial.println("Sensor OK");
  Serial.println("STATUS: OK");

  // Long range mode for desk-to-floor measurement (~700-1200 mm)
  sensor.setDeviceMode(VL53L0X_DEVICEMODE_CONTINUOUS_RANGING, false);
  sensor.startRangeContinuous();
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

  if (sensor.isRangeComplete()) {
    uint16_t distance = sensor.readRangeResult();

    if (distance >= 8190) {
      Serial.println("ERROR: out of range");
    } else {
      Serial.print("distance: ");
      Serial.print(distance);
      Serial.println(" mm");
    }
  }

  delay(100);
}
