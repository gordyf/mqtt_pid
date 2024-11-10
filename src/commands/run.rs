use clap::Parser;
use pid::Pid;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time;

#[derive(Parser, Clone)]
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
    pub async fn run(self: RunArgs, mut pid: Pid<f32>) -> Result<(), Box<dyn std::error::Error>> {
        let mut mqttoptions = MqttOptions::new("mqtt", self.mqtt_host, self.mqtt_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_max_packet_size(2000, 1000);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        client
            .subscribe(self.input_topic, QoS::AtMostOnce)
            .await
            .unwrap();

        let (tx, mut rx) = mpsc::channel(32);

        {
            let client = client.clone();
            let output_topic = self.output_topic.clone();

            task::spawn(async move {
                let mut interval = time::interval(Duration::from_millis(1000));
                let mut last_input_value = 0.0;
                loop {
                    interval.tick().await;

                    while let Ok(value) = rx.try_recv() {
                        last_input_value = value;
                    }

                    let output = pid
                        .next_control_output(last_input_value)
                        .output
                        .clamp(0.0, 100.0);

                    client
                        .publish(
                            output_topic.clone(),
                            QoS::AtLeastOnce,
                            false,
                            output.to_string(),
                        )
                        .await
                        .unwrap();
                }
            });
        }

        loop {
            let notification = eventloop.poll().await.unwrap();
            if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) = notification {
                if let Ok(payload) = std::str::from_utf8(&publish.payload) {
                    if let Ok(value) = payload.parse::<f32>() {
                        tx.send(value).await.unwrap();
                    }
                }
            }
        }
    }
}
