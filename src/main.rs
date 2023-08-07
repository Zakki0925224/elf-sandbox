use std::process::Command;

use clap::Parser;
use sudo::RunningAs;

use crate::args::Arguments;

mod args;

fn exec_command(program: &str, args: &[&str]) {
    let output_result = Command::new(program).args(args).output().unwrap();

    if output_result.status.success() {
        println!("{}", String::from_utf8_lossy(&output_result.stdout));
    } else {
        eprintln!("{}", String::from_utf8_lossy(&output_result.stderr));
    }
}

fn create_container(container_name: &str, distribution: &str, release: &str, arch: &str) {
    println!(
        "Creating container ({}-{}-{})...",
        distribution, release, arch
    );

    exec_command(
        "sudo",
        &[
            "sudo",
            "lxc-create",
            "-t",
            "download",
            "-n",
            container_name,
            "--",
            "-d",
            distribution,
            "-r",
            release,
            "-a",
            arch,
        ],
    );
}

fn start_container(container_name: &str) {
    println!("Starting container...");
    exec_command("sudo", &["lxc-start", "-n", container_name]);
}

fn stop_container(container_name: &str) {
    println!("Stopping container...");
    exec_command("sudo", &["lxc-stop", "-n", container_name]);
}

fn destroy_container(container_name: &str) {
    println!("Destroying container...");
    exec_command("sudo", &["lxc-destroy", "-n", container_name]);
}

fn attach_container(container_name: &str, command: &str) {
    println!("Attaching container (Command: {})...", command);
    exec_command("sudo", &["lxc-attach", "-n", container_name, "--", command]);
}

fn main() {
    match sudo::check() {
        RunningAs::Root => (),
        _ => panic!("You must be run as sudo"),
    }

    let args = Arguments::parse();

    let container_name = "sandbox";
    let distribution = "ubuntu";
    let release = "jammy";
    let arch = "amd64";

    create_container(container_name, distribution, release, arch);
    start_container(container_name);
    attach_container(container_name, "ls");
    stop_container(container_name);
    destroy_container(container_name);
}
