use std::{fs::OpenOptions, io::Write, process::Command, time::Duration};

use wait_timeout::ChildExt;

use crate::mount_entry;

const PV_LXC_PATH: &str = "/var/lib/lxc";
const SYSLOG_PATH: &str = "/var/log/syslog";

enum CommandResult {
    Ok,
    Err,
    TimedOut,
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
    pub mount_root_path: String,
}

impl Container {
    pub fn new(
        container_name: String,
        distribution: String,
        release: String,
        arch: String,
        timeout: u64,
        mount_root_path: String,
    ) -> Self {
        return Self {
            container_name,
            distribution,
            release,
            arch,
            state: ContainerState::NotExist,
            timeout,
            mount_root_path,
        };
    }

    pub fn create(&mut self) {
        match self.state {
            ContainerState::NotExist => (),
            _ => panic!("Container already exists"),
        }

        println!(
            "Creating container ({}-{}-{})...",
            self.distribution, self.release, self.arch
        );

        match self.exec_command(
            "sudo",
            &[
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
            _ => {
                panic!("Failed to create container");
            }
        }
    }

    pub fn set_config(&self, config_str: &str) {
        match self.state {
            ContainerState::Created => (),
            _ => {
                println!(
                    "Container is not right after created, config \"{}\" is not set",
                    config_str
                );
            }
        }

        let mut config = OpenOptions::new()
            .append(true)
            .open(format!("{}/{}/config", PV_LXC_PATH, self.container_name))
            .expect("Failed to open container config file");

        config
            .write_all(config_str.as_bytes())
            .expect("Failed to write to container config file");
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

        println!("Starting container...");

        match self.exec_command("sudo", &["lxc-start", "-n", &self.container_name]) {
            CommandResult::Ok => {
                println!("Started container!");
                self.state = ContainerState::Running;
            }
            _ => {
                println!("Failed to start container");
            }
        }

        println!("Running setup script...");
        //self.attach(&format!("sh /mnt/{}/{}", SEND_DIR_NAME, SETUP_SH_FILE_NAME));
        self.exec_command(
            "sudo",
            &[
                "lxc-attach",
                "-n",
                &self.container_name,
                "--",
                "sh",
                &format!(
                    "{}/{}",
                    self.mount_root_path,
                    mount_entry::SETUP_SH_FILE_NAME
                ),
            ],
        );
    }

    pub fn execute_target(&mut self) {
        self.attach(&format!(
            "cp {}/{} /root/{}",
            self.mount_root_path,
            mount_entry::TARGET_ELF_FILE_NAME,
            mount_entry::TARGET_ELF_FILE_NAME
        ));

        self.exec_command(
            "sudo",
            &[
                "lxc-attach",
                "-n",
                &self.container_name,
                "--",
                "bash",
                "-c",
                &format!("cd /root && ./{}", mount_entry::TARGET_ELF_FILE_NAME),
            ],
        );
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
            CommandResult::TimedOut => {
                self.stop();
                return;
            }
            _ => (),
        }

        println!("Attached!");
    }

    pub fn stop(&mut self) {
        match self.state {
            ContainerState::Running => (),
            ContainerState::Stopped => {
                println!("Container is already stopped");
                return;
            }
            _ => {
                println!("Container is not running");
                return;
            }
        }

        // copy syslog
        // TODO: if failed to copy, enter to loop of container stopping
        self.attach(&format!(
            "cp {} {}/{}",
            SYSLOG_PATH,
            self.mount_root_path,
            mount_entry::SYSLOG_FILE_NAME
        ));

        println!("Stopping container...");

        match self.exec_command("sudo", &["lxc-stop", "-n", &self.container_name]) {
            CommandResult::Ok => {
                println!("Stopped container!");
                self.state = ContainerState::Stopped;
            }
            _ => {
                println!("Failed to stop container");
            }
        }
    }

    pub fn destroy(&mut self) {
        match self.state {
            ContainerState::Stopped | ContainerState::Created => (),
            ContainerState::Running => {
                println!("Container is not stopped");
                return;
            }
            ContainerState::NotExist => {
                println!("Container is not exists");
                return;
            }
        }

        println!("Destroying container...");

        match self.exec_command("sudo", &["lxc-destroy", "-n", &self.container_name]) {
            CommandResult::Ok => {
                println!("Destroyed container!");
                self.state = ContainerState::NotExist;
            }
            _ => {
                println!("Failed to destroy container");
            }
        }
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
                return CommandResult::TimedOut;
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
}
