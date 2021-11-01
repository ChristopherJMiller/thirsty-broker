use super::schema::sensor;

#[derive(Queryable)]
pub struct Sensor {
  pub id: i32,
  pub sensor_id: String,
  pub nickname: String,
  pub dry_reading: Option<i32>,
  pub wet_reading: Optiona<i32>
}

#[derive(Insertable)]
#[table_name="sensor"]
pub struct NewSensor {
  pub sensor_id: String,
  pub nickname: Sting
}