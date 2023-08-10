use std::{
    fs::{self},
    path::Path,
};

use clap::Parser;
use sudo::RunningAs;
use syslog::SyslogEntry;

use crate::args::Arguments;

mod args;
mod rule;
mod syslog;
mod sysmon;

const SYSLOG_FILE_NAME: &str = "syslog";
const TARGETS_DIR_NAME: &str = "targets";
const TARGET_FILE_NAME: &str = "target.elf";

fn main() {
    match sudo::check() {
        RunningAs::Root => (),
        _ => panic!("You must be run as sudo"),
    }

    let args = Arguments::parse();
    let syslog_path = &format!("{}/{}", args.target_root_dir, SYSLOG_FILE_NAME);
    let target_elf_path = &format!(
        "{}/{}/{}",
        args.target_root_dir, TARGETS_DIR_NAME, TARGET_FILE_NAME
    );

    // check directory
    if !Path::new(syslog_path).exists() || !Path::new(target_elf_path).exists() {
        panic!("Detect invalid directory");
    }

    let mut syslog_entries = vec![];

    for line in fs::read_to_string(syslog_path).unwrap().lines() {
        match SyslogEntry::parse(line.to_string()) {
            Some(entry) => syslog_entries.push(entry),
            None => (),
        }
    }

    let mut detection_info = vec![];

    if let Some(info) = rule::mkdir(&syslog_entries) {
        detection_info.push(info);
    }

    if let Some(info) = rule::wget(&syslog_entries) {
        detection_info.push(info);
    }

    if let Some(info) = rule::chmod(&syslog_entries) {
        detection_info.push(info);
    }

    detection_info.extend(rule::rm(&syslog_entries));

    if rule::rm_is(&detection_info, "/var/log") || rule::rm_is(&detection_info, "~/.bash_history") {
        println!("Detected to remove log");
    }

    if rule::wget_and_chmod(&detection_info) {
        println!("Detected creation of wget and chmod processes.");
    }

    println!("{:?}", detection_info);
}
