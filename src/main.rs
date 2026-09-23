use std::process::Command;
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

// convert unix path to Z:\ structure
trait WinePath {
    fn to_wine_path(&self) -> Result<String, Box<dyn Error>> ;
}

impl WinePath for Path {
    fn to_wine_path(&self) -> Result<String, Box<dyn Error>> {
        let abspath = self.canonicalize()
            .map_err(|e| format!("{path}: {e}", path = self.to_string_lossy()))?
            .display()
            .to_string();

        Ok(format!("Z:{}", abspath.replace("/", "\\")))
    }
}

// generate server command in `serverdir` using config in `configdir`
fn server_command(serverdir: String, configdir: &Path) -> Result<Command, Box<dyn Error>> {
    let mut cmd = Command::new("wine");

    let configjson = configdir.join("settings.json").to_wine_path()?;
    let seasonjson = configdir.join("season.json").to_wine_path()?;

    cmd.current_dir(serverdir)
        .arg("AssettoCorsaEVOServer.exe")
        .arg("-configjson")
        .arg(configjson)
        .arg("-seasonjson")
        .arg(seasonjson)
        .arg("-no_lobby");

    Ok(cmd)
}



fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();
    let serverdir = cli.serverdir.unwrap_or(".".to_string());
    match cli.command {
        Commands::Launch(args) => {
            let configpath = Path::new(&args.configdir);
            let mut server = Supervisor::new(server_command(serverdir, &configpath)?);
            return server.run();
        }
    }
}
