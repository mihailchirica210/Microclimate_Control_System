-- MySQL for Arduino Microclimate Control System

CREATE DATABASE IF NOT EXISTS arduino_db;
USE arduino_db;

CREATE TABLE IF NOT EXISTS sensor_data (
    id INT NOT NULL AUTO_INCREMENT,
    temperature FLOAT NOT NULL,
    humidity FLOAT NOT NULL,
    air_quality INT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS users (
    id INT NOT NULL AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL UNIQUE,
    role VARCHAR(20) NOT NULL,
    api_token VARCHAR(255) NOT NULL,
    PRIMARY KEY (id)
);

INSERT IGNORE INTO users (username, role, api_token)
VALUES ('admin', 'admin', 'super_secure_token');
