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
        let args: String = self.command.get_args()
            .map(|x| x.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");

        let prog = self.command.get_program().to_string_lossy();

        println!("Exectuing: {} {}", prog, args);

        let cmd = self.command.spawn();

        let mut child = match cmd {
            Ok(child) => child,
            Err(_) => return,
        };

        let _ = child.wait();
    }
}
