use std::process::Command;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, Args};

use ace_champion::supervisor::Supervisor;

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
    fn to_wine_path(&self) -> String;
}

impl WinePath for PathBuf {
    fn to_wine_path(&self) -> String {
        self.display().to_string().replace("/", "\\")
    }
}

// generate server command in `serverdir` using config in `configdir`
fn server_command(serverdir: String, configdir: PathBuf) -> Command {
    let mut cmd = Command::new("wine");

    let configjson = configdir.join("settings.json").to_wine_path();
    let seasonjson = configdir.join("season.json").to_wine_path();

    cmd.current_dir(serverdir)
        .arg("AssettoCorsaEVOServer.exe")
        .arg("-configjson")
        .arg(configjson)
        .arg("-seasonjson")
        .arg(seasonjson)
        .arg("-no_lobby");

    cmd
}



fn main() {
    let cli = CLI::parse();
    let serverdir = cli.serverdir.unwrap_or(".".to_string());
    match cli.command {
        Commands::Launch(args) => {
            let configpath = Path::new(&args.configdir).canonicalize().unwrap();
            let mut server = Supervisor::new(server_command(serverdir, configpath));
            server.run();
        }
    }
}
