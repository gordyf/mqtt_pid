use clap::Parser;
use pid::Pid;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
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
    pub async fn run(self: RunArgs, mut pid: Pid<f32>) {
        let mut mqttoptions = MqttOptions::new("mqtt", self.mqtt_host, self.mqtt_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_max_packet_size(2000, 1000);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        client
            .subscribe(self.input_topic, QoS::AtMostOnce)
            .await
            .unwrap();

        let last_value: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));

        {
            let last_value = Arc::clone(&last_value);

            task::spawn(async move {
                let mut interval = time::interval(Duration::from_millis(1000));
                loop {
                    let cur_value = *last_value.lock().unwrap();
                    let output = pid.next_control_output(cur_value).output;
                    let output = output.clamp(0.0, 100.0);

                    println!("Current value: {}, PID output: {:?}", cur_value, output);
                    interval.tick().await;
                }
            });
        }

        loop {
            let notification = eventloop.poll().await.unwrap();
            if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) = notification {
                let payload = publish.payload;
                let payload = std::str::from_utf8(&payload).unwrap();
                println!("Received payload: {}", payload);

                let mut last_value = last_value.lock().unwrap();
                *last_value = payload.parse::<f32>().unwrap();
            }
        }
    }
}