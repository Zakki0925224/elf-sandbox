use std::{env, fs, os::fd::AsRawFd};

use lxc::{attach::Options, Container};
use lxc_sys::lxc_groups_t;
use sudo::RunningAs;

fn gen_attach_options() -> Options {
    let options = Options {
        attach_flags: 0,
        namespaces: -1,
        personality: -1,
        initial_cwd: std::ptr::null_mut(),
        uid: 0,
        gid: 0,
        env_policy: 0,
        extra_env_vars: std::ptr::null_mut(),
        extra_keep_env: std::ptr::null_mut(),
        stdin_fd: std::io::stdin().as_raw_fd(),
        stdout_fd: std::io::stdout().as_raw_fd(),
        stderr_fd: std::io::stderr().as_raw_fd(),
        log_fd: std::io::stdout().as_raw_fd(),
        lsm_label: std::ptr::null_mut(),
        groups: lxc_groups_t {
            size: 0,
            list: std::ptr::null_mut(),
        },
    };

    return options;
}

fn run_command(container: &Container, command: &str) {
    let splitted: Vec<&str> = command.split(" ").collect();
    let prog = splitted[0];

    println!("Running command: \"{}\"...", command);
    let result = container.attach_run_wait(&mut gen_attach_options(), prog, &splitted);

    match result {
        Err(e) => println!("Error: {}", e),
        Ok(s) => println!("Ok, waitpid status={}", s),
    }
}

fn main() {
    match sudo::check() {
        RunningAs::Root => (),
        _ => panic!("You must be run as sudo"),
    }

    let container_name = "sandbox";
    let distoribution = "ubuntu";
    let release = "jammy";
    let arch = "amd64";

    println!("LXC version: {}", lxc::version());
    println!(
        "LXC path: {}",
        lxc::get_global_config_item("lxc.lxcpath").unwrap()
    );
    println!("Current path: {}", env::current_dir().unwrap().display());

    let container =
        lxc::Container::new(container_name, None).expect("Failed to setup lxc_container struct");

    if container.is_defined() {
        panic!("Container already exists");
    }

    println!("Creating container...");
    container
        .create(
            "download",
            None,
            None,
            ::lxc::CreateFlags::QUIET,
            &["-d", distoribution, "-r", release, "-a", arch],
        )
        .expect("Failed to create container rootfs");

    // fs::copy(
    //     "./sysmon-setup.sh",
    //     format!(
    //         "{}/{}/rootfs/root/sysmon-setup.sh",
    //         lxc::get_global_config_item("lxc.lxcpath").unwrap(),
    //         container_name
    //     ),
    // )
    // .expect("Failed to copy a file");

    println!("Starting container...");
    container
        .start(false, &[])
        .expect("Failed to start the container");

    //run_command(&container, "/usr/bin/bash ./sysmon-setup.sh");

    println!("Stopping container...");
    container.stop().expect("Failed to kill the container.");

    println!("Destoroying container...");
    container
        .destroy()
        .expect("Failed to destroy the container.");
}
