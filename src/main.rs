use pid::Pid;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::task;
use tokio::time;

async fn async_main() {
    let mut mqttoptions = MqttOptions::new("epyc", "epyc.taileca64.ts.net", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(200000, 1000);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("#", QoS::AtMostOnce).await.unwrap();

    let mut pid = Pid::new(15.0, 100.0);
    pid.p(10.0, 100.0);
    pid.i(1.0, 100.0);
    pid.d(2.0, 100.0);

    task::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(1000));
        loop {
            // generate a random number between 0 and 100
            let random_number = rand::random::<f64>() * 100.0;
            let output = pid.next_control_output(random_number);
            println!("Random number: {}, PID output: {:?}", random_number, output);
            interval.tick().await;
        }
    });

    loop {
        let notification = eventloop.poll().await.unwrap();
        println!("Received = {:?}", notification);
    }
}

#[tokio::main]
async fn main() {
    async_main().await;
}
