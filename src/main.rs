use std::path::{Path, PathBuf};
use std::error::Error;

use clap::{Parser, Subcommand, Args};

mod server;

use server::Supervisor;

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
}

#[derive(Args)]
struct LaunchArgs {
    configdir: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();
    let serverdir = Path::new(&cli.serverdir.unwrap_or(".".to_string())).canonicalize()?;

    match cli.command {
        Commands::Launch(args) => {
            let configpath = Path::new(&args.configdir);
            let mut server = Supervisor::new(server::server_command(&serverdir, &configpath)?);
            return server.run();
        }
    }
}
