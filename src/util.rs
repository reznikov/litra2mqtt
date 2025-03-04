use std::env;
use std::error::Error;
use std::time::Duration;

use regex::Regex;
use rumqttc::{AsyncClient, EventLoop, MqttOptions};

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

    if (host.is_empty() || port == 0) || username.is_empty() || password.is_empty() {
        panic!("MQTT_HOST, MQTT_PORT, MQTT_USERNAME, and MQTT_PASSWORD must be set");
    }

    let mut mqtt_options = MqttOptions::new("litra2mqtt", host, port);

    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_clean_session(true);
    mqtt_options.set_credentials(username, password);

    let (client, event_loop) = AsyncClient::new(mqtt_options, 10);

    Ok((client, event_loop))
}

pub fn get_serial(value: &str) -> Option<String> {
    let re = Regex::new(r"logitech/litra-.*/([^/]+)/([^/]+)/set").unwrap();
    re.captures(value).and_then(|cap| {
        let serial_number = cap.get(1)?.as_str().to_string();
        Some(serial_number)
    })
}

pub fn get_property(value: &str) -> Option<String> {
    let re = Regex::new(r"logitech/litra-.*/([^/]+)/([^/]+)/set").unwrap();
    re.captures(value).and_then(|cap| {
        let state = cap.get(2)?.as_str().to_string();
        Some(state)
    })
}
