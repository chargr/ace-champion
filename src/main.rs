use std::path::{Path, PathBuf};
use std::error::Error;

use clap::{Parser, Subcommand, Args};

mod server;

use server::{ServerProcess, Supervisor};

#[derive(Parser)]
struct CLI {
    #[command(subcommand)]
    command: Commands,

    /// execute server in this directory
    #[arg(short = 'S', long)]
    serverdir: Option<String>,

}

#[derive(Subcommand)]
enum Commands {
    Launch(LaunchArgs),
    Stop(LaunchArgs),
}

#[derive(Args)]
struct LaunchArgs {
    configdir: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();
    let serverdir = Path::new(&cli.serverdir.unwrap_or(".".to_string())).canonicalize()?;

    let result : Result<(), Box<dyn Error>> = match cli.command {
        Commands::Launch(args) => {
            let configpath = Path::new(&args.configdir);
            let mut supervisor = Supervisor::new(
                ServerProcess::new(serverdir, configpath.to_path_buf())?
            );
            supervisor.run()
        },
        Commands::Stop(args) => {
            let configpath = Path::new(&args.configdir);
            let pidfile = configpath.canonicalize()?.join("server.pid");

            server::server_stop(&pidfile)
        },
    };

    result
}
