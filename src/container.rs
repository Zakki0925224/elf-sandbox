use std::process::Command;

enum CommandResult {
    Ok(String),
    Err(String),
}

#[derive(Debug, Clone, Copy)]
pub enum ContainerState {
    NotExist,
    Created,
    Running,
    Stopped,
}

#[derive(Debug)]
pub struct Container {
    container_name: String,
    distribution: String,
    release: String,
    arch: String,
    state: ContainerState,
    timeout: Option<u32>,
}

impl Container {
    pub fn new(
        container_name: String,
        distribution: String,
        release: String,
        arch: String,
        timeout: Option<u32>,
    ) -> Self {
        return Self {
            container_name,
            distribution,
            release,
            arch,
            state: ContainerState::NotExist,
            timeout,
        };
    }

    pub fn state(&self) -> ContainerState {
        return self.state;
    }

    pub fn create(&mut self) {
        match self.state {
            ContainerState::NotExist => (),
            _ => {
                println!("Container already exists");
                return;
            }
        }

        println!(
            "Creating container ({}-{}-{})...",
            self.distribution, self.release, self.arch
        );

        match self.exec_command(
            "sudo",
            &[
                "sudo",
                "lxc-create",
                "-t",
                "download",
                "-n",
                &self.container_name,
                "--",
                "-d",
                &self.distribution,
                "-r",
                &self.release,
                "-a",
                &self.arch,
            ],
        ) {
            CommandResult::Ok(_) => {
                println!("Created container!");
                self.state = ContainerState::Created;
            }
            CommandResult::Err(stderr) => {
                println!("{}", stderr);
                panic!("Failed to create container");
            }
        }
    }

    pub fn start(&mut self) {
        match self.state {
            ContainerState::Created | ContainerState::Stopped => (),
            _ => {
                println!("TODO");
                return;
            }
        }

        println!("Starting container...");

        match self.exec_command("sudo", &["lxc-start", "-n", &self.container_name]) {
            CommandResult::Ok(_) => {
                println!("Started container!");
                self.state = ContainerState::Running;
            }
            CommandResult::Err(stderr) => {
                println!("{}", stderr);
                panic!("Failed to start container");
            }
        }
    }

    pub fn attach(&self, command: &str) {
        match self.state {
            ContainerState::Running => (),
            _ => {
                println!("Container is not running");
                return;
            }
        }

        println!("Attaching with \"{}\"...", command);

        match self.exec_command(
            "sudo",
            &["lxc-attach", "-n", &self.container_name, "--", command],
        ) {
            CommandResult::Ok(stdout) => {
                println!("{}", stdout);
            }
            CommandResult::Err(stderr) => {
                println!("{}", stderr);
                // TODO: destroy container
                return;
            }
        }

        println!("Attached!");
    }

    pub fn stop(&mut self) {
        match self.state {
            ContainerState::Running => (),
            _ => {
                println!("Container is not running");
                return;
            }
        }

        println!("Stopping container...");

        match self.exec_command("sudo", &["lxc-stop", "-n", &self.container_name]) {
            CommandResult::Ok(_) => {
                println!("Stopped container!");
                self.state = ContainerState::Stopped;
            }
            CommandResult::Err(stderr) => {
                println!("{}", stderr);
                panic!("Failed to stop container");
            }
        }
    }

    pub fn destroy(&mut self) {
        match self.state {
            ContainerState::NotExist => {
                println!("Container is not exists");
                return;
            }
            _ => (),
        }

        println!("Destroying container...");

        match self.exec_command("sudo", &["lxc-destroy", "-n", &self.container_name]) {
            CommandResult::Ok(_) => {
                println!("Destroyed container!");
                self.state = ContainerState::NotExist;
            }
            CommandResult::Err(stderr) => {
                println!("{}", stderr);
                panic!("Failed to destroy container");
            }
        }
    }

    fn exec_command(&self, program: &str, args: &[&str]) -> CommandResult {
        let output_result = Command::new(program).args(args).output().unwrap();

        return match output_result.status.success() {
            true => CommandResult::Ok(String::from_utf8_lossy(&output_result.stdout).to_string()),
            false => CommandResult::Err(String::from_utf8_lossy(&output_result.stderr).to_string()),
        };
    }
}
