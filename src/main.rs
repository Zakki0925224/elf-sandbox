use std::{env, fs, os::fd::AsRawFd};

use clap::Parser;
use lxc::{attach::Options, Container};
use lxc_sys::lxc_groups_t;
use sudo::RunningAs;

use crate::args::Arguments;

mod args;

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

fn wait_container(container: &Container, state: &str) {
    println!("Waiting the container...");
    container.wait(state, 10).expect("Timed out");
}

fn create_container(
    container_name: &str,
    distoribution: &str,
    release: &str,
    arch: &str,
) -> Container {
    let container =
        lxc::Container::new(container_name, None).expect("Failed to setup lxc_container struct");

    if container.is_defined() {
        panic!("The container already exists");
    }

    println!(
        "Creating the container ({}-{}-{})...",
        distoribution, release, arch
    );
    container
        .create(
            "download",
            None,
            None,
            ::lxc::CreateFlags::QUIET,
            &["-d", distoribution, "-r", release, "-a", arch],
        )
        .expect("Failed to create the container rootfs");

    return container;
}

fn start_container(container: &Container) {
    container
        .set_config_item("lxc.cgroup.memory.limit_in_bytes", "256MB")
        .expect("Failed to set config");

    println!("Starting the container...");
    container
        .start(true, &[])
        .expect("Failed to start the container");

    wait_container(&container, "RUNNING");

    if container.state() != "RUNNING" {
        panic!("The container state is not RUNNING");
    }
}

fn stop_container(container: &Container) {
    println!("Stopping the container...");
    container.stop().expect("Failed to kill the container.");
    wait_container(container, "STOPPED");
}

fn destroy_container(container: &Container) {
    println!("Destoroying the container...");
    container
        .destroy()
        .expect("Failed to destroy the container.");
}

fn run_command(container: &Container, command: &str) {
    if container.state() == "STOPPED" {
        println!("Skipping run a command...");
    }

    wait_container(container, "RUNNING");

    let splitted: Vec<&str> = command.split(" ").collect();
    let prog = splitted[0];

    println!("Running a command: \"{}\"...", command);
    let result = container.attach_run_wait(&mut gen_attach_options(), prog, &splitted);

    match result {
        Err(e) => eprintln!("Error: {}", e),
        Ok(s) => println!("Ok, waitpid status={}", s),
    }
}

fn main() {
    match sudo::check() {
        RunningAs::Root => (),
        _ => panic!("You must be run as sudo"),
    }

    let args = Arguments::parse();

    let container_name = "sandbox";
    let distoribution = "ubuntu";
    let release = "jammy";
    let arch = "amd64";
    let lxc_path = lxc::get_global_config_item("lxc.lxcpath").unwrap();

    let log = lxc::Log {
        name: "sndbox".to_string(),
        lxcpath: lxc_path.clone(),
        file: "sandbox.log".to_string(),
        level: lxc::log::Level::Debug,
        prefix: "".to_string(),
        quiet: false,
    };

    log.init().expect("Failed to initialize log");

    println!("LXC version: {}", lxc::version());
    println!("LXC path: {}", lxc_path);
    println!("Current path: {}", env::current_dir().unwrap().display());

    let container = create_container(container_name, distoribution, release, arch);
    start_container(&container);

    // println!("Container state: {}", container.state());
    // println!("Container PID: {}", container.init_pid());
    // println!("Interfaces: {:?}", container.get_interfaces());

    // fs::copy(
    //     "./sysmon-setup.sh",
    //     format!("{}/{}/rootfs/sysmon-setup.sh", lxc_path, container_name),
    // )
    // .expect("Failed to copy a file");

    // run_command(&container, "chmod 100 ./sysmon-setup.sh");
    // run_command(&container, "./sysmon-setup.sh");

    // fs::copy(
    //     args.target_elf_path,
    //     format!("{}/{}/rootfs/target", lxc_path, container_name),
    // )
    // .expect("Failed to copy a file");

    // run_command(&container, "./target");
    run_command(&container, "id");
    run_command(&container, "id");

    // if container.shutdown(30).is_err() {
    //     println!("Failed to cleanly shutdown the container, forcing.");
    //     container.stop().expect("Failed to kill the container.");
    // }

    stop_container(&container);
    destroy_container(&container);

    lxc::Log::close();
}
