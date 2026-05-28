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

pub fn signing_key_from_pkcs8_pem(pem: &str) -> ed25519_dalek::SigningKey {
    let b64 = pem
        .trim()
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");

    let der =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64.as_bytes()).unwrap();

    let seed_bytes: &[u8; 32] = der[16..48].try_into().unwrap();

    ed25519_dalek::SigningKey::from_bytes(seed_bytes)
}

pub fn mask_hex_key(key: &str) -> String {
    let key = key.trim();

    let (prefix, rest) = if let Some(stripped) = key.strip_prefix("0x") {
        ("0x", stripped)
    } else {
        ("", key)
    };

    if rest.len() > 8 {
        if prefix == "0x" {
            format!("0x{}...{}", &rest[..6], &rest[rest.len() - 4..])
        } else {
            format!("{}...{}", &rest[..4], &rest[rest.len() - 4..])
        }
    } else {
        key.to_string()
    }
}

pub fn remove_trailing_zeros(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }

    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_string()
}
