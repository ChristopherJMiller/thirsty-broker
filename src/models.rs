use serde::{Deserialize, Serialize};

use super::schema::sensor;

#[derive(Queryable, Serialize)]
pub struct Sensor {
  pub id: i32,
  pub sensor_id: String,
  pub nickname: String,
  pub dry_reading: Option<i32>,
  pub wet_reading: Option<i32>,
  pub current_reading: Option<i32>,
}

#[derive(Deserialize, Clone)]
pub struct SensorUpdateJSON {
  pub id: i32,
  nickname: Option<String>,
  dry_reading: Option<i32>,
  wet_reading: Option<i32>,
}

impl Into<UpdateSensor> for SensorUpdateJSON {
  fn into(self) -> UpdateSensor {
    UpdateSensor {
      nickname: self.nickname,
      wet_reading: self.wet_reading,
      dry_reading: self.dry_reading,
    }
  }
}

#[derive(Insertable)]
#[table_name = "sensor"]
pub struct NewSensor<'a> {
  pub sensor_id: &'a str,
  pub nickname: &'a str,
  pub current_reading: i32,
}

#[derive(AsChangeset)]
#[table_name = "sensor"]
pub struct UpdateSensor {
  pub nickname: Option<String>,
  pub wet_reading: Option<i32>,
  pub dry_reading: Option<i32>,
}
