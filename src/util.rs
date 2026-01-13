use std::env;
use std::error::Error;
use std::time::Duration;

use regex::Regex;
use rumqttc::{AsyncClient, EventLoop, LastWill, MqttOptions, QoS};

pub fn create_async_client() -> Result<(AsyncClient, EventLoop), Box<dyn Error>> {
    for (key, value) in env::vars() {
        println!("ENV: {key}: {value}");
    }

    let host = env::var("MQTT_HOST").expect("MQTT_HOST must be set");
    let port = env::var("MQTT_PORT")
        .expect("MQTT_PORT must be set")
        .parse()?;
    let username = env::var("MQTT_USERNAME").expect("MQTT_USERNAME must be set");
    let password = env::var("MQTT_PASSWORD").expect("MQTT_PASSWORD must be set");
    let client_id = env::var("MQTT_CLIENT_ID").unwrap_or_else(|_| "litra2mqtt".to_string());

    if (host.is_empty() || port == 0) || username.is_empty() || password.is_empty() {
        panic!("MQTT_HOST, MQTT_PORT, MQTT_USERNAME, and MQTT_PASSWORD must be set");
    }

    let mut mqtt_options = MqttOptions::new(client_id, host, port);

    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_clean_session(false); // Changed to false for better reconnection
    mqtt_options.set_credentials(username, password);
    
    // Set LWT - will be published when client disconnects ungracefully
    let lwt_topic = "litra_bridge/status".to_string();
    let lwt = LastWill::new(
        lwt_topic,
        "offline",
        QoS::AtLeastOnce,
        true, // retain
    );
    mqtt_options.set_last_will(lwt);

    let (client, event_loop) = AsyncClient::new(mqtt_options, 10);

    Ok((client, event_loop))
}

pub fn get_serial(value: &str) -> Option<String> {
    // Updated regex to match new topic structure: litra_bridge/<device_id>/command
    let re = Regex::new(r"litra_bridge/([^/]+)/command").unwrap();
    re.captures(value).and_then(|cap| {
        let device_id = cap.get(1)?.as_str().to_string();
        Some(device_id)
    })
}

pub fn get_instance_id() -> String {
    env::var("LITRA_INSTANCE_ID").unwrap_or_else(|_| {
        uuid::Uuid::new_v4().to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_serial_new_format() {
        assert_eq!(
            get_serial("litra_bridge/device123/command"),
            Some("device123".to_string())
        );
    }

    #[test]
    fn test_get_serial_no_match() {
        assert_eq!(get_serial("invalid/topic"), None);
    }
}
