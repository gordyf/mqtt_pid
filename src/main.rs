mod commands;

use clap::Parser;
use clap::Subcommand;
use commands::run::RunArgs;
use commands::simulate::SimulateArgs;
use pid::Pid;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    setpoint: f32,

    #[arg(long, allow_hyphen_values = true)]
    kp: f32,

    #[arg(long, allow_hyphen_values = true)]
    ki: f32,

    #[arg(long, allow_hyphen_values = true)]
    kd: f32,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Run(RunArgs),
    Simulate(SimulateArgs),
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut pid = Pid::new(args.setpoint, 100.0);
    pid.p(args.kp, 100.0);
    pid.i(args.ki, 100.0);
    pid.d(args.kd, 100.0);

    match args.command.clone().unwrap() {
        Commands::Run(command) => command.run(pid).await,
        Commands::Simulate(command) => command.run(pid).await,
    }
    .unwrap();
}
