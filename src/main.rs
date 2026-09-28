use std::path::{Path, PathBuf};
use std::error::Error;

use nix::sys::signal::{kill};
use nix::unistd::Pid;
use nix::sys::signal::{SIGTERM, SIGHUP};

use clap::{Parser, Subcommand, Args};

mod server;

use server::{ServerProcess, Supervisor};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[command(flatten)]
        common: CommonArgs,

        #[command(flatten)]
        start: StartArgs,
    },
    Stop(CommonArgs),
    Restart(CommonArgs),
}

#[derive(Args)]
struct StartArgs {
    /// change to dir before executing AssettoCorsaServer.exe
    #[arg(short = 'S', long)]
    server_dir: Option<String>,
}

#[derive(Args)]
struct CommonArgs{
    /// directory containing server config and output
    config_dir: PathBuf,
}

impl CommonArgs {
    fn pid_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self.config_dir.canonicalize()?.join("server.pid"))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let result : Result<(), Box<dyn Error>> = match cli.command {
        Commands::Start{common , start} => {
            let install_dir = Path::new(&start.server_dir.unwrap_or(".".to_string())).canonicalize()?;
            let mut supervisor = Supervisor::new(
                ServerProcess::new(install_dir, common.config_dir)?
            );
            supervisor.run()
        },
        Commands::Stop(args) => {
            let pidfile = args.pid_path()?;
            let pid = std::fs::read_to_string(&pidfile)?.trim().parse::<i32>()?;
            kill(Pid::from_raw(pid), SIGTERM)?;
            Ok(())
        },
        Commands::Restart(args) => {
            let pidfile = args.pid_path()?;
            let pid = std::fs::read_to_string(&pidfile)?.trim().parse::<i32>()?;
            kill(Pid::from_raw(pid), SIGHUP)?;
            Ok(())
        }
    };

    result
}
