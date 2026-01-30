# Microclimate Control System

This project is a simple microclimate control system based on **Arduino Uno** and a **Rust backend server**.

Arduino collects environmental data using sensors and sends it to the backend.  
The backend processes the data, validates it, logs events, and decides whether to turn the fan on or off.

---

## 📌 Project Components

### Hardware
- **Arduino Uno**
- **DHT11 sensor** – temperature and humidity
- **MQ-135 sensor** – air quality (ppm)
- **Fan** – used as an actuator for climate control

### Software
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
