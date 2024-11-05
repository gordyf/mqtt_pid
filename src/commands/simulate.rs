use clap::Parser;
use pid::Pid;

#[derive(Parser, Clone)]
pub struct SimulateArgs {
    #[arg(short, long)]
    input_file: String,

    #[arg(short, long)]
    output_file: String,
}

impl SimulateArgs {
    pub async fn run(self: SimulateArgs, mut pid: Pid<f32>) {}
}
