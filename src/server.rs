use std::process::Command;
use std::error::Error;
use std::path::Path;

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
}

impl Supervisor {
    pub fn new(command: Command) -> Self {
        Self {
            command,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let args: String = self.command.get_args()
            .map(|x| x.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        let prog = self.command.get_program().to_string_lossy();

        println!("Exectuing: {} {}", prog, args);

        let mut child = self.command.spawn()?;

        let _ = child.wait()?;
        Ok(())
    }
}
