use std::process::Command;
use std::error::Error;

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
