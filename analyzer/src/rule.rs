use chrono::{DateTime, FixedOffset};

use crate::{syslog::SyslogEntry, sysmon::SysmonEventId};

#[derive(Debug, Eq, PartialEq)]
pub enum Code {
    Mkdir,
    Wget,
    Chmod,
    Rm(Vec<String>),
    FileDelete(String), // target
}

#[derive(Debug)]
pub struct DetectionInfo {
    pub event_id: SysmonEventId,
    pub time_created: DateTime<FixedOffset>,
    pub reason_for_detection: String,
    pub code: Code,
}

pub fn mkdir(entries: &Vec<SyslogEntry>) -> Option<DetectionInfo> {
    for e in entries {
        if let Some(a) = e.sysmon_event.event_data.get("Image") {
            if a.contains("/rootfs/usr/bin/mkdir") {
                return Some(DetectionInfo {
                    event_id: e.sysmon_event.event_id.clone(),
                    time_created: e.sysmon_event.time_created,
                    reason_for_detection: "Created mkdir process".to_string(),
                    code: Code::Mkdir,
                });
            }
        }
    }

    return None;
}

pub fn wget(entries: &Vec<SyslogEntry>) -> Option<DetectionInfo> {
    for e in entries {
        if let Some(a) = e.sysmon_event.event_data.get("Image") {
            if a.contains("/rootfs/usr/bin/wget") {
                let cmd_line = e.sysmon_event.event_data.get("CommandLine").unwrap();
                return Some(DetectionInfo {
                    event_id: e.sysmon_event.event_id.clone(),
                    time_created: e.sysmon_event.time_created,
                    reason_for_detection: format!(
                        "Created wget process (Command Line: {})",
                        cmd_line
                    ),
                    code: Code::Wget,
                });
            }
        }
    }

    return None;
}

pub fn chmod(entries: &Vec<SyslogEntry>) -> Option<DetectionInfo> {
    for e in entries {
        if let Some(a) = e.sysmon_event.event_data.get("Image") {
            if a.contains("/rootfs/usr/bin/chmod") {
                return Some(DetectionInfo {
                    event_id: e.sysmon_event.event_id.clone(),
                    time_created: e.sysmon_event.time_created,
                    reason_for_detection: "Created chmod process".to_string(),
                    code: Code::Chmod,
                });
            }
        }
    }

    return None;
}

pub fn rm(entries: &Vec<SyslogEntry>) -> Vec<DetectionInfo> {
    let mut info = vec![];

    for e in entries {
        if let Some(a) = e.sysmon_event.event_data.get("Image") {
            if a.contains("/rootfs/usr/bin/rm") {
                if let Some(cmd_line) = e.sysmon_event.event_data.get("CommandLine") {
                    let mut s_cmd_line: Vec<String> =
                        cmd_line.split(" ").map(|s| s.to_string()).collect();
                    s_cmd_line.remove(0);

                    info.push(DetectionInfo {
                        event_id: e.sysmon_event.event_id.clone(),
                        time_created: e.sysmon_event.time_created,
                        reason_for_detection: "Created rm process".to_string(),
                        code: Code::Rm(s_cmd_line),
                    });
                }
            }
        }
    }

    return info;
}

pub fn wget_and_chmod(info: &Vec<DetectionInfo>) -> bool {
    return info.iter().find(|i| i.code == Code::Wget).is_some()
        && info.iter().find(|i| i.code == Code::Chmod).is_some();
}

pub fn rm_is(info: &Vec<DetectionInfo>, path: &str) -> bool {
    return info
        .iter()
        .find(|i| match &i.code {
            Code::Rm(args) => args.iter().find(|a| a.contains(path)).is_some(),
            _ => false,
        })
        .is_some();
}

pub fn event_id_23(entries: &Vec<SyslogEntry>) -> Vec<DetectionInfo> {
    let mut info = vec![];

    for e in entries {
        if e.sysmon_event.event_id == SysmonEventId::FILE_DELETE {
            if let Some(target_file_name) = e.sysmon_event.event_data.get("TargetFilename") {
                info.push(DetectionInfo {
                    event_id: e.sysmon_event.event_id.clone(),
                    time_created: e.sysmon_event.time_created,
                    reason_for_detection: "File deleted".to_string(),
                    code: Code::FileDelete(target_file_name.clone()),
                });
            }
        }
    }

    return info;
}

pub fn file_deleted_at(info: &Vec<DetectionInfo>, path: &str) -> bool {
    return info
        .iter()
        .find(|i| match &i.code {
            Code::FileDelete(target) => target.contains(path),
            _ => false,
        })
        .is_some();
}
