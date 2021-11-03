-- Your SQL goes here
CREATE TABLE sensor (
  id SERIAL PRIMARY KEY,
  sensor_id VARCHAR NOT NULL UNIQUE,
  nickname TEXT NOT NULL,
  dry_reading INTEGER,
  wet_reading INTEGER,
  current_reading INTEGER
);
