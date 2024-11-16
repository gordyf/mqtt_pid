mod commands;

use clap::Parser;
use clap::Subcommand;
use commands::run::RunArgs;
use commands::simulate::SimulateArgs;
use pid::Pid;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    setpoint: f64,

    #[arg(long, allow_hyphen_values = true)]
    kp: f64,

    #[arg(long, allow_hyphen_values = true)]
    ki: f64,

    #[arg(long, allow_hyphen_values = true)]
    kd: f64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Run(RunArgs),
    Simulate(SimulateArgs),
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args = Args::parse();
    let mut pid = Pid::<f64>::new(args.setpoint, 100.0);
    pid.p(args.kp, 100.0);
    pid.i(args.ki, 100.0);
    pid.d(args.kd, 100.0);

    match args.command {
        Commands::Run(command) => command.run(pid).await.unwrap(),
        Commands::Simulate(command) => command.run(pid).await.unwrap(),
    };
}
