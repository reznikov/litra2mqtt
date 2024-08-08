use std::error::Error;
use std::time::Duration;

use regex::Regex;
use rumqttc::{AsyncClient, EventLoop, MqttOptions};

pub fn create_async_client() -> Result<(AsyncClient, EventLoop), Box<dyn Error>> {
    let host = dotenv::var("MQTT_HOST").unwrap();
    let port = dotenv::var("MQTT_PORT").unwrap().parse()?;
    let username = dotenv::var("MQTT_USERNAME").unwrap();
    let password = dotenv::var("MQTT_PASSWORD").unwrap();

    let mut mqtt_options = MqttOptions::new("logitech-litra", host, port);

    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_clean_session(true);
    mqtt_options.set_credentials(username, password);

    let (client, event_loop) = AsyncClient::new(mqtt_options, 10);

    return Ok((client, event_loop));
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
