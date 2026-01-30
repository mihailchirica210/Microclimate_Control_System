mod services;

use actix_web::{post, get, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_files;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use tracing_subscriber;
use uuid::Uuid;
use services::secret;
use sqlx::mysql::MySqlPool;

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

#[post("/report")]
async fn report(
    data: web::Json<SensorData>,
    req: HttpRequest,
    thresholds: web::Data<Thresholds>,
    db_pool: web::Data<MySqlPool>,
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

    if let Err(e) = sqlx::query!(
        "INSERT INTO sensor_data (temperature, humidity, air_quality) VALUES (?, ?, ?)",
        data.temperature,
        data.humidity,
        data.air_quality
    )
    .execute(db_pool.get_ref())
    .await
    {
        error!(?correlation_id, "DB insert failed: {}", e);
        return HttpResponse::InternalServerError().body("DB error");
    }

    info!(
        ?correlation_id,
        temp = data.temperature,
        hum = data.humidity,
        aq = data.air_quality,
        "Receiving sensor data"
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
async fn current(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let rec = sqlx::query!(
        "SELECT temperature, humidity, air_quality FROM sensor_data ORDER BY id DESC LIMIT 1"
    )
    .fetch_one(db_pool.get_ref())
    .await;

    match rec {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(_) => HttpResponse::Ok().json(serde_json::json!({"temperature":0.0,"humidity":0.0,"air_quality":0})),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();

    info!("Starting Arduino backend (HTTP + MySQL)");

    // Loading API token
    match secret::get_secret("api/token").await {
        Ok(token) => info!(token = ?token, "API token loaded"),
        Err(e) => error!("Failed to load API token: {}", e),
    }

    // MySQL
    let db_url = "mysql://username:password@localhost/arduino_db";
    let db_pool = MySqlPool::connect(db_url).await.unwrap();

    let thresholds = web::Data::new(Thresholds {
        temp: TEMP_THRESHOLD,
        hum: HUM_THRESHOLD,
        aq: AQ_THRESHOLD,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(thresholds.clone())
            .app_data(web::Data::new(db_pool.clone()))
            .service(report)
            .service(status)
            .service(current)
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
