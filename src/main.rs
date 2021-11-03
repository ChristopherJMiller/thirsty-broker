#[macro_use]
extern crate diesel;
extern crate dotenv;

#[macro_use] 
extern crate rocket;

use lazy_static::lazy_static;
use models::{Sensor, SensorUpdateJSON, UpdateSensor};
use std::sync::Mutex;
use std::thread;
use serde::Deserialize;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use crate::mqtt::run_mqtt;
use rocket::serde::json::Json;
use crate::schema::sensor::dsl::{sensor, dry_reading, wet_reading};
use prometheus::{Opts, Registry, GaugeVec, TextEncoder, Encoder};

pub mod schema;
pub mod models;
pub mod mqtt;

lazy_static! {
  static ref PSQL_CONN: Mutex<Option<PgConnection>> = Mutex::new(None);
}

#[get("/")]
fn no_features() -> &'static str {
  "Hello, world! Please enable features such as broker and metrics to increase functionality."
}

#[get("/")]
fn get_sensors() -> Json<Vec<Sensor>> {
  let sensors = sensor.select(sensor.default_selection()).load::<Sensor>(PSQL_CONN.lock().unwrap().as_ref().unwrap()).unwrap();
  Json(sensors)
}

#[post("/update", format = "application/json", data = "<sensor_json>")]
fn update_sensor(sensor_json: Json<SensorUpdateJSON>) -> Json<Sensor> {
  let changeset: UpdateSensor = sensor_json.clone().into();
  let result = diesel::update(sensor.find(sensor_json.id))
    .set(&changeset)
    .get_result::<Sensor>(PSQL_CONN.lock().unwrap().as_ref().unwrap()).unwrap();

  Json(result)
}

#[get("/metrics")]
fn get_metrics() -> String {
  let sensors = sensor.filter(dry_reading.is_not_null()).load::<Sensor>(PSQL_CONN.lock().unwrap().as_ref().unwrap()).unwrap();
  let sensor_names: Vec<&str> = sensors.iter().map(|x| x.sensor_id.as_str()).collect();

  let opts = Opts::new("Sensors", "Adjusted Values from the Sensor Readings");
  let gauge = GaugeVec::new(opts, &sensor_names).unwrap();

  let r = Registry::new();
  r.register(Box::new(gauge.clone())).unwrap();
  
  for s in sensors.iter() {
    if let Some(value) = s.current_reading {
      gauge.with_label_values(&[s.sensor_id.as_str()]).set(value.into());
    }
  }

  let mut buffer = vec![];
  let encoder = TextEncoder::new();
  let metric_families = r.gather();
  encoder.encode(&metric_families, &mut buffer).unwrap();

  String::from_utf8(buffer).unwrap()
}

fn load_psql(database_url: String) {
  let conn = PgConnection::establish(&database_url)
    .expect(&format!("Error connecting to {}", database_url));

  *PSQL_CONN.lock().unwrap() = Some(conn);
}

#[rocket::main]
async fn main() {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  let mqtt_url = env::var("MQTT_URL").expect("MQTT_URL must be set");
  load_psql(database_url);

  #[cfg(feature = "broker")]
  thread::spawn(|| {
    run_mqtt(mqtt_url, &PSQL_CONN);
  });

  let routes = if cfg!(feature = "metrics") && cfg!(feature = "broker") {
    routes![get_sensors, update_sensor, get_metrics]
  } else if cfg!(feature = "broker") {
    routes![get_sensors, update_sensor]
  } else if cfg!(feature = "metrics") {
    routes![get_sensors, get_metrics]
  } else {
    routes![no_features]
  };

  let _ = rocket::build().mount("/", routes).launch().await;
}

