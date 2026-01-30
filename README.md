# Microclimate Control System

This project is a simple microclimate control system based on **Arduino Uno**.

Arduino collects environmental data using sensors and sends it to the backend.  
The backend processes the data, validates it, logs events, and decides whether to turn the fan on or off.

---

## 📌 Project Components

### Hardware Components

1. **Arduino Elegoo UNO R3 (with USB cable connected to the laptop)**  
   Acts as the main microcontroller unit. It reads data from the sensors, processes it, and controls the connected actuators.

2. **DHT11 Sensor**  
   Measures ambient temperature and humidity. The collected data is transmitted to the Arduino for further processing.

3. **Fan Blade with 3–6V DC Motor (with wires)**  
   Activated when air quality deteriorates or humidity exceeds the defined safe thresholds, helping to stabilize the microclimate.

4. **LCD 1602 Module (with pin header)**  
   Displays real-time environmental data, allowing the user to monitor temperature, humidity, and system status directly.

5. **830 Tie-Points Breadboard**  
   Used for assembling the circuit and connecting the Arduino with sensors, display, and other components without soldering.

6. **Three LEDs (red, yellow, and green) with three 220 Ω resistors**  
   Provide visual alerts indicating the system state:
   - Green: normal conditions  
   - Yellow: warning level  
   - Red: critical conditions exceeding safe thresholds  

### Software Components

- **Rust** (backend server)
- **Actix-web** – HTTP server
- **Serde** – JSON serialization/deserialization
- **Tracing** – structured logging
- **UUID** – correlation IDs for logs

---

## 📊 Controlled Parameters

The system operates within the following thresholds:

- **Temperature:** 24–27 °C  
- **Humidity:** 40–50 %  
- **Air quality:** MQ-135 < 100 ppm  

If any value exceeds the threshold, the backend sends a command to turn the fan on.

---

## 🔧 Backend Features

- HTTP API for receiving sensor data
- Input validation and protection against malformed data
- Role-based access control (Arduino / Admin)
- Centralized structured logging with correlation IDs
- Clear error handling without exposing sensitive data
- Configurable thresholds
- Simple and readable code suitable for educational purposes

---

## 📡 API Endpoints

### `POST /report`
Receives sensor data from Arduino.

**Example request:**
```json
{
  "temperature": 26.5,
  "humidity": 45.0,
  "air_quality": 80
}
