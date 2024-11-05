use pid::Pid;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::task;
use tokio::time;

async fn async_main() {
    let mut mqttoptions = MqttOptions::new("epyc", "epyc.taileca64.ts.net", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(200000, 1000);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("air/pm25", QoS::AtMostOnce).await.unwrap();

    let mut pid = Pid::new(2.0, 100.0);
    pid.p(-5.0, 100.0);
    pid.i(-0.50, 100.0);
    pid.d(-1.0, 100.0);

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

#[tokio::main]
async fn main() {
    async_main().await;
}
