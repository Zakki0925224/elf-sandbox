use clap::Parser;
use container::Container;
use sudo::RunningAs;

use crate::args::Arguments;

mod args;
mod container;

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
    //container.attach("bash");
    container.stop();
    container.analyze_syslog();
    container.destroy();
}
