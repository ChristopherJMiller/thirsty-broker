#[macro_use]
extern crate diesel;
extern crate dotenv;

#[macro_use] 
extern crate rocket;

use rumqtt::{MqttClient, MqttOptions, QoS, Notification};
use std::{thread, time::Duration};
use thirsty_support::{ControllerMessage, ProbePort};
use std::str;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

#[launch]
fn rocket() -> _ {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
  PgConnection::establish(&database_url)
    .expect(&format!("Error connecting to {}", database_url));

  let mqtt_options = MqttOptions::new("thisty-broker", "localhost", 1883);
  let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();
    
  mqtt_client.subscribe("thirsty/data", QoS::AtLeastOnce).unwrap();
  let sleep_time = Duration::from_secs(1);
  thread::spawn(move || {
    for i in 0..100 {
      let payload = ControllerMessage {
        controller_id: "test".to_string(),
        probe: ProbePort::J1,
        value: i
      };
      thread::sleep(sleep_time);
      mqtt_client.publish("thirsty/data", QoS::AtLeastOnce, false, serde_json::to_string(&payload).unwrap()).unwrap();
    }
  });

  for notification in notifications {
    match notification {
      Notification::Publish(publish) => {
        let message: ControllerMessage = serde_json::from_str(&str::from_utf8(&publish.payload).unwrap()).unwrap();
        println!("Message: {:?}", message);
      }
      _ => {}
    }
  }

  rocket::build().mount("/", routes![index])
}
