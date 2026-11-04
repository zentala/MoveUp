/**
 * tof_reader.ino
 *
 * Reads distance from VL53L0X Time-of-Flight sensor
 * via I2C on Seeed XIAO ESP32-C3 or Waveshare RP2040-Zero.
 *
 * XIAO ESP32-C3 Grove I2C pinout (verified by I2C scan):
 *   SDA = GPIO6 (D4)
 *   SCL = GPIO7 (D5)
 * RP2040-Zero: SDA = GP26, SCL = GP27 (I2C1); VIN = 3V3, GND = GND.
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

#if defined(ARDUINO_ARCH_RP2040)
#define I2C_SDA 26
#define I2C_SCL 27
#define SENSOR_WIRE Wire1
#define DESK_BOARD_NAME "RP2040-Zero"
#else
#define I2C_SDA 6
#define I2C_SCL 7
#define SENSOR_WIRE Wire
#define DESK_BOARD_NAME "XIAO ESP32-C3"
#endif

Adafruit_VL53L0X sensor;
bool sensorReady = false;
uint32_t lastSensorAttempt = 0;
char command[32];
size_t commandLength = 0;
bool commandOverflow = false;

void initialiseSensor() {
  lastSensorAttempt = millis();
  sensorReady = sensor.begin(VL53L0X_I2C_ADDR, false, &SENSOR_WIRE,
                            Adafruit_VL53L0X::VL53L0X_SENSE_LONG_RANGE);
  if (sensorReady) sensorReady = sensor.startRangeContinuous(100);
  Serial.println(sensorReady ? "STATUS: OK" : "ERROR: VL53L0X not found or failed to start");
}

void setup() {
  Serial.begin(115200);
  // USB CDC must remain usable even when no serial monitor is open.
  const uint32_t started = millis();
  while (!Serial && millis() - started < 2000) delay(10);

  Serial.println(DEVICE_ID);
  Serial.println("VL53L0X Distance Sensor — " DESK_BOARD_NAME);

#if defined(ARDUINO_ARCH_RP2040)
  SENSOR_WIRE.setSDA(I2C_SDA);
  SENSOR_WIRE.setSCL(I2C_SCL);
  SENSOR_WIRE.begin();
#else
  Wire.begin(I2C_SDA, I2C_SCL);
#endif

  initialiseSensor();
}

void handleSerial() {
  while (Serial.available()) {
    const char next = Serial.read();
    if (next == '\r') continue;
    if (next == '\n') {
      command[commandLength] = '\0';
      if (!commandOverflow && strcmp(command, "PING") == 0) {
        Serial.println(DEVICE_ID);
      } else if (!commandOverflow && strcmp(command, "STATUS") == 0) {
        Serial.println("BOARD: " DESK_BOARD_NAME);
        Serial.print("I2C: SDA=");
        Serial.print(I2C_SDA);
        Serial.print(" SCL=");
        Serial.println(I2C_SCL);
        Serial.println(sensorReady ? "STATUS: OK" : "ERROR: VL53L0X not found or failed to start");
      }
      commandLength = 0;
      commandOverflow = false;
    } else if (commandLength < sizeof(command) - 1) {
      command[commandLength++] = next;
    } else {
      commandOverflow = true;
    }
  }
}

void loop() {
  handleSerial();

  if (!sensorReady) {
    if (millis() - lastSensorAttempt >= 3000) initialiseSensor();
    delay(10);
    return;
  }

  if (sensor.isRangeComplete()) {
    uint16_t distance = sensor.readRangeResult();

    if (sensor.readRangeStatus() != 0 || distance >= 8190) {
      Serial.println("ERROR: out of range");
    } else {
      Serial.print("distance: ");
      Serial.print(distance);
      Serial.println(" mm");
    }
  }

  delay(100);
}
