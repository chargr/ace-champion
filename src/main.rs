use std::process::Command;

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
    configdir: String
}

// generate server command in `serverdir` using config in `configdir`
fn server_command(serverdir: String, configdir: String) -> Command {
    let mut cmd = Command::new("wine");

    cmd.current_dir(serverdir)
        .arg("AssettoCorsaEVOServer.exe")
        .arg("-configjson")
        .arg(format!("{configdir}/settings.json"))
        .arg("-seasonjson")
        .arg(format!("{configdir}/season.json"))
        .arg("-no_lobby");

    cmd
}

fn main() {
    let cli = CLI::parse();
    let serverdir = cli.serverdir.unwrap_or(".".to_string());
    match cli.command {
        Commands::Launch(args) => {
            let mut server = Supervisor::new(server_command(serverdir, args.configdir));
            server.run();
        }
    }
}
