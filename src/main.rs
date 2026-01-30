mod services;

use actix_web::{post, get, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_files;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use tracing_subscriber;
use uuid::Uuid;
use services::secret;
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Deserialize)]
struct SensorData {
    temperature: f32,
    humidity: f32,
    air_quality: u32,
}

impl SensorData {
    fn validate(&self) -> Result<(), &'static str> {
        if !(-50.0..=150.0).contains(&self.temperature) {
            return Err("temperature out of range");
        }
        if !(0.0..=100.0).contains(&self.humidity) {
            return Err("humidity out of range");
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct ControlCommand {
    fan_on: bool,
}

struct Thresholds {
    temp: f32,
    hum: f32,
    aq: u32,
}

// Default thresholds
const TEMP_THRESHOLD: f32 = 27.0;
const HUM_THRESHOLD: f32 = 50.0;
const AQ_THRESHOLD: u32 = 100;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Role {
    Admin,
    Arduino,
}

fn get_role_from_header(req: &HttpRequest) -> Option<Role> {
    req.headers()
        .get("X-Role")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_lowercase())
        .and_then(|s| match s.as_str() {
            "admin" => Some(Role::Admin),
            "arduino" | "device" => Some(Role::Arduino),
            _ => None,
        })
}

// Shared data for dashboard
struct SharedData {
    temperature: f32,
    humidity: f32,
    air_quality: u32,
}

lazy_static! {
    static ref CURRENT_DATA: Mutex<SharedData> = Mutex::new(SharedData {
        temperature: 0.0,
        humidity: 0.0,
        air_quality: 0,
    });
}

#[post("/report")]
async fn report(
    data: web::Json<SensorData>,
    req: HttpRequest,
    thresholds: web::Data<Thresholds>,
) -> impl Responder {
    let correlation_id = Uuid::new_v4();

    if get_role_from_header(&req) != Some(Role::Arduino) {
        error!("CorrelationID {}: Unauthorized access", correlation_id);
        return HttpResponse::Forbidden().body("Unauthorized");
    }

    if let Err(e) = data.validate() {
        error!(?correlation_id, "Validation failed: {}", e);
        return HttpResponse::BadRequest().body(e);
    }

    // Save latest sensor data
    {
        let mut current = CURRENT_DATA.lock().unwrap();
        current.temperature = data.temperature;
        current.humidity = data.humidity;
        current.air_quality = data.air_quality;
    }

    info!(
        ?correlation_id,
        temp = data.temperature,
        hum = data.humidity,
        aq = data.air_quality,
        "Received sensor data"
    );

    let fan_on = data.temperature > thresholds.temp
        || data.humidity > thresholds.hum
        || data.air_quality > thresholds.aq;

    info!(?correlation_id, fan_on, "Fan turns on");

    HttpResponse::Ok().json(ControlCommand { fan_on })
}

#[get("/status")]
async fn status(req: HttpRequest) -> impl Responder {
    let correlation_id = Uuid::new_v4();
    if get_role_from_header(&req).is_none() {
        return HttpResponse::Forbidden().body("Unauthorized");
    }
    info!(?correlation_id, "Status checked");
    HttpResponse::Ok().body("Running with thresholds")
}

#[get("/current")]
async fn current_data() -> impl Responder {
    let current = CURRENT_DATA.lock().unwrap();
    HttpResponse::Ok().json(&*current)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("info")
        .init();

    info!("Starting Arduino backend (HTTP)");

    match secret::get_secret("api/token").await {
        Ok(token) => info!(token = ?token, "API token loaded"),
        Err(e) => error!("Failed to load API token: {}", e),
    }

    let thresholds = web::Data::new(Thresholds {
        temp: TEMP_THRESHOLD,
        hum: HUM_THRESHOLD,
        aq: AQ_THRESHOLD,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(thresholds.clone())
            .service(report)
            .service(status)
            .service(current_data)
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
