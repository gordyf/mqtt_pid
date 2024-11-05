use std::io::BufRead;

use clap::Parser;
use pid::Pid;

#[derive(Parser, Clone)]
pub struct SimulateArgs {
    #[arg(short, long)]
    input_file: String,
}

impl SimulateArgs {
    pub async fn run(self: SimulateArgs, mut pid: Pid<f32>) {
        let file = std::fs::File::open(self.input_file).unwrap();
        let reader = std::io::BufReader::new(file);
        let lines = reader.lines();

        for line in lines {
            let line = line.unwrap();
            let value: f32 = line.parse().unwrap();

            let output = pid.next_control_output(value).output;
            let output = output.clamp(0.0, 100.0);

            println!("{},{}", value, output);
        }
    }
}
