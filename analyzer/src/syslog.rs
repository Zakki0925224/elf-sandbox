use regex::Regex;

use crate::sysmon::SysmonEvent;

#[derive(Debug, Clone)]
pub struct SyslogEntry {
    pub log: String,
    pub sysmon_event: SysmonEvent,
}

impl SyslogEntry {
    pub fn parse(log: String) -> Option<Self> {
        let regex =
            Regex::new(r"[A-Z][a-z]{2}\s+\d+\s\d\d:\d\d:\d\d\s\S+\ssysmon\S*:\s(.+)").unwrap();

        match regex.is_match(&log) {
            true => (),
            false => return None,
        };

        let captures = regex.captures(&log).unwrap();

        // parse
        return match SysmonEvent::from_xml(&captures[1]) {
            Ok(event) => Some(Self {
                sysmon_event: event,
                log,
            }),
            Err(_) => None,
        };
    }
}
