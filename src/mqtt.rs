extern crate diesel;

use diesel::PgConnection;
use rumqtt::{MqttClient, MqttOptions, Notification, QoS};
use thirsty_support::ControllerMessage;
use std::{str, sync::Mutex};
use crate::models::{NewSensor, Sensor};
use crate::schema::sensor::dsl::{sensor, sensor_id, current_reading};
use self::diesel::prelude::*;

fn get_sensor_id(message: &ControllerMessage) -> String {
  let probe: String = message.probe.into();
  format!("{}_{}", message.controller_id, probe)
}

pub fn run_mqtt(mqtt_url: String, conn: &Mutex<Option<PgConnection>>) {

  let mqtt_options = MqttOptions::new("thirsty-broker", mqtt_url, 1883);
  let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();

  mqtt_client.subscribe("thirsty/data", QoS::AtLeastOnce).unwrap();
  for notification in notifications {
    match notification {
      Notification::Publish(publish) => {
        if let Ok(conn) = conn.try_lock() {
          let message: ControllerMessage = serde_json::from_str(&str::from_utf8(&publish.payload).unwrap()).unwrap();
          println!("Message Received: {:?}", message);
          let sensor_id_string = get_sensor_id(&message);
          let existing_sensor = sensor.filter(sensor_id.eq(sensor_id_string.clone()))
            .load::<Sensor>(conn.as_ref().unwrap())
            .expect("Failed to get sensors. DB down?");

          // Create sensor if it's missing
          if existing_sensor.len() == 0 {
            let new_sensor = NewSensor {
              sensor_id: sensor_id_string.as_str(),
              nickname: sensor_id_string.as_str(),
              current_reading: i32::from(message.value as u16)
            };

            diesel::insert_into(crate::schema::sensor::table)
              .values(&new_sensor)
              .get_result::<Sensor>(conn.as_ref().unwrap())
              .expect("Failed to save new sensor");
          } else {
            diesel::update(sensor.find(existing_sensor[0].id))
              .set(current_reading.eq(i32::from(message.value as u16)))
              .get_result::<Sensor>(conn.as_ref().unwrap())
              .expect("Unable to update sensor reading");
          }
        }

      }
      _ => {}
    }
  }
}
