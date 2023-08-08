use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    process::Command,
    time::Duration,
};

use wait_timeout::ChildExt;

const SEND_DIR_NAME: &str = "send";
const SETUP_SH_FILE_NAME: &str = "setup.sh";
const TARGET_ELF_FILE_NAME: &str = "target.elf";

enum CommandResult {
    Ok,
    Err,
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
    timeout: u64,
    setup_sh_path: String,
    target_elf_path: String,
}

impl Container {
    pub fn new(
        container_name: String,
        distribution: String,
        release: String,
        arch: String,
        timeout: u64,
        setup_sh_path: String,
        target_elf_path: String,
    ) -> Self {
        return Self {
            container_name,
            distribution,
            release,
            arch,
            state: ContainerState::NotExist,
            timeout,
            setup_sh_path,
            target_elf_path,
        };
    }

    pub fn create(&mut self) {
        match self.state {
            ContainerState::NotExist => (),
            _ => {
                println!("Container already exists");
                return;
            }
        }

        println!("Creating sender pack...");
        self.create_send_pack();

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
            CommandResult::Ok => {
                println!("Created container!");
                self.state = ContainerState::Created;
            }
            CommandResult::Err => {
                panic!("Failed to create container");
            }
        }
    }

    pub fn start(&mut self) {
        match self.state {
            ContainerState::Created | ContainerState::Stopped => (),
            ContainerState::NotExist => {
                println!("Container is not exist");
                return;
            }
            ContainerState::Running => {
                println!("Container is running");
                return;
            }
        }

        // set config
        let mut config = OpenOptions::new()
            .append(true)
            .open(format!("/var/lib/lxc/{}/config", self.container_name))
            .expect("Failed to open config file");

        config
            .write_all(
                format!(
                    "lxc.mount.entry = {}/{} mnt/{} none bind,create=dir 0 0",
                    env::current_dir().unwrap().display(),
                    SEND_DIR_NAME,
                    SEND_DIR_NAME
                )
                .as_bytes(),
            )
            .unwrap();

        println!("Starting container...");

        match self.exec_command("sudo", &["lxc-start", "-n", &self.container_name]) {
            CommandResult::Ok => {
                println!("Started container!");
                self.state = ContainerState::Running;
            }
            CommandResult::Err => {
                panic!("Failed to start container");
            }
        }

        println!("Running setup script...");
        self.attach(&format!("chmod -R 100 /mnt/{}", SEND_DIR_NAME));
        self.attach(&format!("sh /mnt/{}/{}", SEND_DIR_NAME, SETUP_SH_FILE_NAME));
    }

    pub fn execute_target(&mut self) {
        self.attach(&format!("/mnt/{}/{}", SEND_DIR_NAME, TARGET_ELF_FILE_NAME));
    }

    pub fn attach(&mut self, command: &str) {
        match self.state {
            ContainerState::Running => (),
            _ => {
                println!("Container is not running");
                return;
            }
        }

        println!("Attaching with \"{}\"...", command);

        let mut args = vec!["lxc-attach", "-n", &self.container_name, "--"];
        args.extend(command.split(" "));

        match self.exec_command("sudo", &args) {
            CommandResult::Ok => (),
            CommandResult::Err => {
                // TODO: destroy container
                self.stop();
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
            CommandResult::Ok => {
                println!("Stopped container!");
                self.state = ContainerState::Stopped;
            }
            CommandResult::Err => {
                println!("Failed to stop container");
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
            CommandResult::Ok => {
                println!("Destroyed container!");
                self.state = ContainerState::NotExist;
            }
            CommandResult::Err => {
                println!("Failed to destroy container");
            }
        }

        self.remove_send_pack();
    }

    fn exec_command(&self, program: &str, args: &[&str]) -> CommandResult {
        let mut child = Command::new(program).args(args).spawn().unwrap();

        let status_code = match child
            .wait_timeout(Duration::from_secs(self.timeout))
            .unwrap()
        {
            Some(status) => status.code(),
            None => {
                println!("Timed out {} secs", self.timeout);
                child.kill().unwrap();
                child.wait().unwrap().code()
            }
        };

        return match status_code {
            Some(code) => {
                if code == 0 {
                    CommandResult::Ok
                } else {
                    CommandResult::Err
                }
            }
            None => CommandResult::Err,
        };
    }

    fn create_send_pack(&self) {
        match fs::create_dir(SEND_DIR_NAME) {
            Ok(()) => (),
            Err(_) => {
                // regenerate
                fs::remove_dir_all(SEND_DIR_NAME).unwrap();
                fs::create_dir(SEND_DIR_NAME).unwrap();
            }
        }

        // setup sh
        fs::copy(
            &self.setup_sh_path,
            format!("{}/{}", SEND_DIR_NAME, SETUP_SH_FILE_NAME),
        )
        .expect("Failed to copy a file");

        // target elf
        fs::copy(
            &self.target_elf_path,
            format!("{}/{}", SEND_DIR_NAME, TARGET_ELF_FILE_NAME),
        )
        .expect("Failed to copy a file");
    }

    fn remove_send_pack(&self) {
        fs::remove_dir_all(SEND_DIR_NAME).expect("Failed to remove a directory");
    }
}
