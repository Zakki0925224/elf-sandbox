use std::fs;

use uuid::Uuid;

use crate::{container::Container, mount_entry};

const ANALYSIS_RESULT_DIR_PATH: &str = "./analysis_results";

#[derive(Debug)]
pub struct Analyzer {
    uuid: Uuid,
    container: Container,
    setup_sh_path: String,
    target_elf_path: String,
    mount_dir_path: String,
}

impl Analyzer {
    pub fn new(
        container_name: String,
        distribution: String,
        release: String,
        arch: String,
        timeout: u64,
        setup_sh_path: String,
        target_elf_path: String,
        mount_dir_path: String,
    ) -> Self {
        return Self {
            uuid: Uuid::new_v4(),
            container: Container::new(
                container_name,
                distribution,
                release,
                arch,
                timeout,
                "/mnt/sandtmp".to_string(),
            ),
            setup_sh_path,
            target_elf_path,
            mount_dir_path,
        };
    }

    pub fn analyze(&mut self) {
        let mut mount_root_path = self.container.mount_root_path.clone();

        if mount_root_path.starts_with("/") {
            mount_root_path.remove(0);
        }

        self.generate_mount_entries();

        self.container.create();
        self.container.set_config(&format!(
            "lxc.mount.entry = {} {} none bind,create=dir 0 0",
            self.mount_dir_path, mount_root_path
        ));
        self.container.start();
        self.container.execute_target();
        self.container.stop();
        self.container.destroy();

        self.generate_analysis_result();
        self.remove_mount_entries();
    }

    fn generate_mount_entries(&self) {
        fs::create_dir_all(&self.mount_dir_path).expect("Failed to create mount directory");
        fs::copy(
            &self.setup_sh_path,
            format!(
                "{}/{}",
                self.mount_dir_path,
                mount_entry::SETUP_SH_FILE_NAME
            ),
        )
        .expect("Failed to copy setup sh file");
        fs::copy(
            &self.target_elf_path,
            format!(
                "{}/{}",
                self.mount_dir_path,
                mount_entry::TARGET_ELF_FILE_NAME
            ),
        )
        .expect("Failed to copy target elf file");
    }

    fn remove_mount_entries(&self) {
        fs::remove_dir_all(&self.mount_dir_path).expect("Failed to remove mount direcotry");
    }

    fn generate_analysis_result(&self) {
        let result_dir_path = &format!("{}/{}", ANALYSIS_RESULT_DIR_PATH, self.uuid.to_string());

        fs::create_dir_all(&format!("{}/targets", result_dir_path))
            .expect("Failed to create result directory");
        fs::copy(
            &format!("{}/{}", self.mount_dir_path, mount_entry::SYSLOG_FILE_NAME),
            &format!("{}/{}", result_dir_path, mount_entry::SYSLOG_FILE_NAME),
        )
        .expect("Failed to copy syslog file");
        fs::copy(
            &format!(
                "{}/{}",
                self.mount_dir_path,
                mount_entry::TARGET_ELF_FILE_NAME
            ),
            &format!(
                "{}/targets/{}",
                result_dir_path,
                mount_entry::TARGET_ELF_FILE_NAME
            ),
        )
        .expect("Failed to copy target elf file");
    }
}
