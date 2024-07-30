use std::error::Error;
use std::time::Duration;

use rumqttc::{AsyncClient, EventLoop, MqttOptions};

pub fn create_async_client() -> Result<(AsyncClient, EventLoop), Box<dyn Error>> {
    let host = dotenv::var("MQTT_PORT").unwrap();
    let port = dotenv::var("MQTT_PORT").unwrap().parse()?;
    let username = dotenv::var("MQTT_USERNAME").unwrap();
    let password = dotenv::var("MQTT_PASSWORD").unwrap();

    let mut mqtt_options = MqttOptions::new("logitech_litra", host, port);

    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_clean_session(true);
    mqtt_options.set_credentials(username, password);

    let (client, event_loop) = AsyncClient::new(mqtt_options, 10);

    return Ok((client, event_loop));
}
