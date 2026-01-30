mod services;

use actix_web::{post, get, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use tracing_subscriber;
use uuid::Uuid;
use services::secret;

#[derive(Deserialize)]
struct SensorData {
    temperature: f32,
    humidity: f32,
    air_quality: u32,
}

impl SensorData {
    fn validate(&self) -> Result<(), &'static str> {
        if !( -50.0..=150.0 ).contains(&self.temperature) {
            return Err("Temperature out of range");
        }
        if !(0.0..=100.0).contains(&self.humidity) {
            return Err("Humidity out of range");
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct ControlCommand {
    fan_on: bool,
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

fn main_role(req: &HttpRequest) -> Option<Role> {
    req
        .headers()
        .get("X-Role")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_lowercase())
        .and_then(|s| match s.as_str() {
            "admin" => Some(Role::Admin),
            "arduino" | "device" => Some(Role::Arduino),
            _ => None,
        })
}

#[post("/report")]
async fn report(
    data: web::Json<SensorData>,
    req: HttpRequest,
) -> impl Responder {
    let correlation_id = Uuid::new_v4();
    if main_role(&req) != Some(Role::Arduino) {
        error!("CorrelationID {}: Unauthorized access", correlation_id);
        return HttpResponse::Forbidden().body("Unauthorized");
    }

    if let Err(e) = data.validate() {
        error!(?correlation_id, "Validation failed: {}", e);
        return HttpResponse::BadRequest().body(e);
    }

    info!(?correlation_id, temp = data.temperature, hum = data.humidity, aq = data.air_quality, "Received sensor data");

    let fan_on =
        data.temperature > TEMP_THRESHOLD ||
        data.humidity > HUM_THRESHOLD ||
        data.air_quality > AQ_THRESHOLD;

    info!(?correlation_id, fan_on, "Fan turns on");

    HttpResponse::Ok().json(ControlCommand { fan_on })
}

#[get("/status")]
async fn status(req: HttpRequest) -> impl Responder {
    let correlation_id = Uuid::new_v4();
    if main_role(&req).is_none() {
        return HttpResponse::Forbidden().body("Unauthorized");
    }
    info!(?correlation_id, "Status checked");
    HttpResponse::Ok().body("Running with thresholds")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("info")
        .init();

    info!("Starting Arduino backend (Http)");

    match secret::get_secret("api/token").await {
        Ok(token) => info!(token = ?token, "API token loaded"),
        Err(e) => error!("Failed to load API token: {}", e),
    }

    HttpServer::new(move || {
        App::new()
            .service(report)
            .service(status)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}