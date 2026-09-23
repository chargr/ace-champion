use std::process::Command;

pub struct Supervisor {
    command: Command, 
}

impl Supervisor {
    pub fn new(command: Command) -> Self {
        Self {
            command,
        }
    }

    pub fn run(&mut self) {
        let cmd = self.command.spawn();

        let mut child = match cmd {
            Ok(child) => child,
            Err(_) => return,
        };

        let _ = child.wait();
    }
}
