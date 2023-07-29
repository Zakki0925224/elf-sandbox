use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum ExitStatus {
    Successful,
    Unsuccessful(i32),
}

pub fn exec_cmd(args: &[&str]) -> ExitStatus {
    println!("Running lxc args...: {:?}", args);

    let output = Command::new("lxc")
        .args(args)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stdout.len() > 0 {
        println!("stdout=================:");
        println!("{}", stdout);
    }

    if stderr.len() > 0 {
        println!("stderr=================:");
        println!("{}", stderr);
    }

    let code = output.status.code().unwrap();

    return match code {
        0 => ExitStatus::Successful,
        c => ExitStatus::Unsuccessful(c),
    };
}
