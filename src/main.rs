use sudo::RunningAs;

fn main() {
    match sudo::check() {
        RunningAs::Root => (),
        _ => panic!("You must be run as sudo"),
    }

    let container_name = "sandbox";
    let distoribution = "ubuntu";
    let release = "bionic";
    let arch = "amd64";

    let c =
        lxc::Container::new(container_name, None).expect("Failed to setup lxc_container struct");

    if c.is_defined() {
        panic!("Container already exists");
    }

    println!("Creating container...");
    c.create(
        "download",
        None,
        None,
        ::lxc::CreateFlags::QUIET,
        &["-d", distoribution, "-r", release, "-a", arch],
    )
    .expect("Failed to create container rootfs");

    println!("Starting container...");
    c.start(false, &[]).expect("Failed to start the container");

    println!("Stopping container...");
    c.stop().expect("Failed to kill the container.");

    println!("Destoroying container...");
    c.destroy().expect("Failed to destroy the container.");
}
