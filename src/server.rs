use std::process::Command;
use std::error::Error;
use std::path::{Path, PathBuf};

use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{BufReader, BufRead};
use std::process::Stdio;

use nix::unistd::{fork, ForkResult};

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
pub fn server_command(serverdir: &Path, configdir: &Path) -> Result<Command, Box<dyn Error>> {
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

pub struct Supervisor {
    command: Command,
    pidfile: PathBuf,
    logfile: PathBuf,
}

impl Supervisor {
    pub fn new(command: Command, pidfile: PathBuf, logfile: PathBuf) -> Self {
        Self {
            command,
            pidfile,
            logfile,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {

        // gather up original commands for display
        let prog = self.command.get_program().to_string_lossy().to_string();
        let args: String = self.command.get_args()
            .map(|x| x.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        let cwd = match self.command.get_current_dir() {
            Some(path) => path.to_string_lossy().to_string(),
            None => "(unset)".to_string(),
        };

        match unsafe{fork()?} {
            ForkResult::Parent { child } => {
                let mut file = File::create(&self.pidfile)?;
                writeln!(file, "{}", child.as_raw())?;
                drop(file);

                println!("forked supervisor to {}", child);

                Ok(())
            },
            ForkResult::Child => {
                nix::unistd::setsid()?;
                // create and open log file
                let mut log = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&self.logfile)?;

                // setup pipe to capture and log
                let (reader, writer) = std::io::pipe()?;

                self.command
                    .stdin(Stdio::null())
                    .stderr(writer.try_clone()?)
                    .stdout(writer);

                writeln!(log, "--- server start ---")?;
                writeln!(log, "Executing: {} {} in {}", prog, args, cwd)?;

                let mut server = self.command.spawn()?;

                self.command
                    .stderr(Stdio::null())
                    .stdout(Stdio::null());

                let mut reader = BufReader::new(reader);
                let mut buf = Vec::new();

                while reader.read_until(b'\n', &mut buf)? > 0 {
                    let line = String::from_utf8_lossy(&buf);
                    write!(log, "{line}")?;
                    buf.clear();
                }

                for line in BufReader::new(reader).lines().map_while(Result::ok) {
                    let _ = writeln!(log, "{line}");
                }

                server.wait()?;
                writeln!(log, "--- server stop ---")?;

                std::process::exit(0)
            },
        }
    }
}
