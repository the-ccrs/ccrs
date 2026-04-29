pub fn get_env_as_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .map(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => default,
        })
        .unwrap_or(default)
}

pub fn get_env_as_number<T>(name: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<T>().ok())
        .unwrap_or(default)
}

pub fn get_env_as_string(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

pub fn convert_unix_timestamp_milliseconds_to_timestamp(
    unix_timestamp_milliseconds: i64,
) -> chrono::DateTime<chrono::Utc> {
    match chrono::TimeZone::timestamp_millis_opt(&chrono::Utc, unix_timestamp_milliseconds) {
        chrono::LocalResult::Single(datetime) => datetime,
        _ => panic!(
            "Invalid unix timestamp in milliseconds: {}",
            unix_timestamp_milliseconds
        ),
    }
}
