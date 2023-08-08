use clap::Parser;
use container::Container;
use sudo::RunningAs;

use crate::args::Arguments;

mod args;
mod container;

fn main() {
    match sudo::check() {
        RunningAs::Root => panic!("You must not be run as sudo"),
        _ => (),
    }

    let args = Arguments::parse();

    let container_name = "sandbox";
    let distribution = "ubuntu";
    let release = "jammy";
    let arch = "amd64";

    let mut container = Container::new(
        container_name.to_string(),
        distribution.to_string(),
        release.to_string(),
        arch.to_string(),
        args.timeout,
        args.setup_sh_path,
        args.target_elf_path,
    );

    container.create();
    container.start();
    container.execute_target();
    container.attach("/usr/bin/bash");
    container.stop();
    container.destroy();
}
