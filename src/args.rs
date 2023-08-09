use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(long)]
    pub setup_sh_path: String,
    #[arg(long)]
    pub target_elf_path: String,
    #[arg(long)]
    pub mount_dir_path: String,
    #[arg(long)]
    pub timeout: u64,
}
