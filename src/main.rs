mod lxc;

fn main() {
    lxc::exec_cmd(&[
        "launch",
        "images:ubuntu/22.04",
        "ubuntu-sandbox",
        "--vm",
        "-c",
        "security.secureboot=false",
    ]);

    lxc::exec_cmd(&["shell", "ubuntu-sandbox"]);
    lxc::exec_cmd(&["stop", "ubuntu-sandbox"]);
    lxc::exec_cmd(&["delete", "ubuntu-sandbox"]);
    lxc::exec_cmd(&["list"]);
}
