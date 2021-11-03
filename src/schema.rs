table! {
    sensor (id) {
        id -> Int4,
        sensor_id -> Varchar,
        nickname -> Text,
        dry_reading -> Nullable<Int4>,
        wet_reading -> Nullable<Int4>,
        current_reading -> Nullable<Int4>,
    }
}
