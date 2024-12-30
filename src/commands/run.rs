use clap::Parser;
use log::debug;
use log::error;
use log::info;
use pid::Pid;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use std::time::Instant;
use tokio::sync::watch;
use tokio::task;
use tokio::time;

#[derive(Parser)]
pub struct RunArgs {
    #[arg(short, long)]
    input_topic: String,

    #[arg(short, long)]
    output_topic: String,

    #[arg(long)]
    mqtt_host: String,

    #[arg(long, default_value = "1883")]
    mqtt_port: u16,
}

impl RunArgs {
    pub async fn run(self: RunArgs, mut pid: Pid<f64>) -> Result<(), Box<dyn std::error::Error>> {
        let mut mqttoptions = MqttOptions::new("mqtt_pid", self.mqtt_host, self.mqtt_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_max_packet_size(2000, 1000);
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        if let Err(e) = client.subscribe(self.input_topic, QoS::AtMostOnce).await {
            error!("Failed to subscribe to MQTT topic: {}", e);
            return Err(Box::new(e));
        }

        let (tx, mut rx) = watch::channel(0);

        {
            let client = client.clone();
            let output_topic = self.output_topic.clone();

            task::spawn(async move {
                let mut interval = time::interval(Duration::from_millis(1000));
                let mut last_output_value: Option<u8> = None;
                loop {
                    interval.tick().await;

                    let last_input_value = *rx.borrow_and_update();

                    let output = pid
                        .next_control_output(last_input_value as f64)
                        .output
                        .clamp(0.0, 100.0)
                        .round() as u8;

                    if last_output_value != Some(output) {
                        debug!("Emitting new output value: {}", output);

                        match client
                            .publish(
                                output_topic.clone(),
                                QoS::AtLeastOnce,
                                false,
                                output.to_string(),
                            )
                            .await
                        {
                            Err(e) => error!("Failed to publish to MQTT topic: {}", e),
                            Ok(_) => last_output_value = Some(output),
                        }
                    }
                }
            });
        }

        info!("Topic subscribed; waiting for events.");

        let ping_timeout_duration = Duration::from_secs(30);
        let mut last_ping_response = Instant::now();

        loop {
            let elapsed = last_ping_response.elapsed();
            if elapsed >= ping_timeout_duration {
                panic!("No MQTT ping response received within the timeout period. Exiting.");
            }

            match time::timeout(ping_timeout_duration, eventloop.poll()).await {
                Ok(notification) => match notification {
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                        if let Ok(payload) = std::str::from_utf8(&publish.payload) {
                            if let Ok(value) = payload.parse::<u16>() {
                                debug!("Received new input value: {}", value);
                                if let Err(e) = tx.send(value) {
                                    error!("Failed to send via channel: {}", e);
                                }
                            }
                        }
                    }
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::PingResp)) => {
                        last_ping_response = Instant::now();
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("MQTT error: {:?}", e);
                    }
                },
                Err(_) => continue,
            }
        }
    }
}
