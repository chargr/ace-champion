use std::process::{Command,Child};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufRead};
use std::process::Stdio;
use std::thread;

use nix::unistd::{fork, ForkResult};

use nix::unistd::Pid;
use nix::sys::signal::{kill, Signal};
use signal_hook::iterator::Signals;
use signal_hook::consts::{SIGINT, SIGTERM};

use simplelog::{WriteLogger,LevelFilter};
use log::{error, info, warn};

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
        .arg(seasonjson);

    Ok(cmd)
}

pub fn server_stop(pidfile: &Path) -> Result<(), Box<dyn Error>> {
    let pid = Pid::from_raw(std::fs::read_to_string(pidfile)?.trim().parse::<i32>()?);
    kill(pid, nix::sys::signal::SIGTERM)?;
    Ok(())
}

pub struct ServerProcess {
    install_dir: PathBuf, // install directory containing server binaries
    server_dir: PathBuf, // path containing config and and outputs
    pid_path: PathBuf, // path container our (parent) process pid
    log_path: PathBuf,
    child: Option<Child>, //
}

impl ServerProcess {
    pub fn new(install_dir: PathBuf, server_dir: PathBuf) -> Result<Self, Box<dyn Error>> {

        let pid_path = server_dir.canonicalize()?.join("server.pid");
        let log_path = server_dir.canonicalize()?.join("server.log");

        let child = None;

        Ok(Self {
            install_dir,
            server_dir,
            pid_path,
            log_path,
            child,
        })
    }

    fn log_path(&self) -> &Path {
        &self.log_path
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let configjson = self.server_dir.join("settings.json").to_wine_path()?;
        let seasonjson = self.server_dir.join("season.json").to_wine_path()?;

        let (reader, writer) = std::io::pipe()?;

        let mut command = Command::new("wine");
        command
            .current_dir(&self.install_dir)
            .arg("AssettoCorsaEVOServer.exe")
            .arg("-no_lobby")
            .arg("-configjson")
            .arg(configjson)
            .arg("-seasonjson")
            .arg(seasonjson)
            .stdin(Stdio::null())
            .stderr(writer.try_clone()?)
            .stdout(writer);

        // gather up original commands for display
        let prog = command.get_program().to_string_lossy().to_string();
        let args: String = command.get_args()
            .map(|x| x.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        let cwd = match command.get_current_dir() {
            Some(path) => path.to_string_lossy().to_string(),
            None => "(unset)".to_string(),
        };

        log::info!("--- server start ---");
        log::info!("executing: {} {} in {}", prog, args, cwd);

        let child = match command.spawn() {
            Ok(server) => server,
            Err(e) => {
                let msg = format!("Unable to execute {prog}: {e}");
                log::info!("{msg}");
                return Err(msg.into());
            }
        };

        // record child to kill when request
        self.child = Some(child);

        // start a thread to read to log file
        thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
            // read lines into log
            let mut reader = BufReader::new(reader);
            let mut buf = Vec::new();

            while reader.read_until(b'\n', &mut buf)? > 0 {
                let line = String::from_utf8_lossy(&buf);
                log::info!("{}", line.trim_end());
                buf.clear();
            }

            Ok(())
        });

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        log::info!("--- server starting ---");
        self.run()
    }

    pub fn wait(&mut self) -> Result<(), Box<dyn Error>> {
        let Some(mut child) = self.child.take() else { return Ok(()); };
        let _ = child.wait();
        Ok(())
    }
    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        let Some(mut child) = self.child.take() else { return Ok(()) };
        log::info!("--- server stopping ---");
        let pid = Pid::from_raw(i32::try_from(child.id())?);
        kill(pid, Signal::SIGINT)?;
        child.wait()?;
        Ok(())
    }

    pub fn restart(&mut self) -> Result<(), Box<dyn Error>> {
        self.stop()?;
        self.start()?;
        Ok(())
    }
}

pub struct Supervisor {
    process: ServerProcess,
}

impl Supervisor {
    pub fn new(process: ServerProcess) -> Self {
        Self {
            process,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {

        match unsafe{fork()?} {
            ForkResult::Parent { child } => {
                println!("forked to {}", child);
                let mut file = File::create(&self.process.pid_path)?;
                writeln!(file, "{}", child.as_raw())?;
                drop(file);
                Ok(())
            },
            ForkResult::Child => {
                nix::unistd::setsid()?;
                WriteLogger::init(
                        LevelFilter::Info,
                        simplelog::Config::default(),
                        File::options().create(true).append(true).open(&self.process.log_path())?)?;

                //signal thread
                let mut signals = Signals::new([SIGINT, SIGTERM])?;

                self.process.start()?;

                for sig in signals.forever() {
                    match sig {
                        SIGINT | SIGTERM => {
                            let _ =  self.process.stop();
                            break;
                        }
                        _ => {},
                    }
                }

                log::logger().flush();
                std::process::exit(0)
            },
        }
    }
}
