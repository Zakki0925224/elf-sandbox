use clap::Parser;
use sandbox::Sandbox;
use sudo::RunningAs;

use crate::args::Arguments;

mod args;
mod container;
mod sandbox;

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

    let mut analyzer = Sandbox::new(
        container_name.to_string(),
        distribution.to_string(),
        release.to_string(),
        arch.to_string(),
        args.timeout,
        args.setup_sh_path,
        args.target_elf_path,
        args.mount_dir_path,
    );

    analyzer.run_container();
}
