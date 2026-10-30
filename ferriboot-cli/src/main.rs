use std::fs::{self, create_dir_all};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::bootentry::LOADER_PATH;
use crate::esp::find_esp;
use clap::{Parser, Subcommand};
mod bootentry;
mod cmd_install;
mod cmd_mkconfig;
mod cmd_uninstall;
mod cmdline;
mod esp;
mod hooks;
mod install;
mod kernels;
mod mkconfig;
mod status;
mod uki;
/// UEFI bootloader for Linux: install and configure ferriboot
#[derive(Parser)]
#[command(
    version,
    about,
    after_help = "install, mkconfig and uninstall need root: sudo ferriboot <command>"
)]
struct Cli {
    /// show what would be done without changing anything
    #[arg(long, global = true)]
    dry_run: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// install the bootloader to the ESP, generate the config, register a
    /// firmware boot entry, copy ferriboot to /usr/local/bin and install
    /// the kernel update hook
    Install {
        /// bootloader image to install
        #[arg(long, default_value = "/usr/lib/ferriboot/ferriboot.efi")]
        #[arg(value_name = "PATH")]
        efi: PathBuf,
    },

    /// find kernels, copy them to the ESP if needed and write
    /// EFI/ferriboot/ferriboot.conf
    Mkconfig {
        /// do nothing unless ferriboot is installed on the ESP
        #[arg(long)]
        if_installed: bool,
    },

    /// remove the boot entry, EFI/ferriboot on the ESP, the kernel update
    /// hook and /usr/local/bin/ferriboot
    Uninstall,

    /// show whether ferriboot is installed and how the last boot went;
    Status,
}

fn installed_by_package(exe: &Path) -> bool {
    let Ok(res) = fs::canonicalize(exe) else {
        return false;
    };

    res.starts_with("/usr/bin")
}

fn die(msg: &str) -> ! {
    eprintln!("ferriboot: {}", msg);
    std::process::exit(1);
}
fn require_root() {
    let Ok(meta) = fs::metadata("/proc/self") else {
        die("cannot check user id (/proc/self)")
    };

    let uid = meta.uid();
    if uid != 0 {
        die("must be run as root (try: sudo ferriboot <command>)")
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Install { efi } => cmd_install::cmd_install(&efi, cli.dry_run),
        Command::Mkconfig { if_installed } => cmd_mkconfig::cmd_mkconfig(cli.dry_run, if_installed),
        Command::Uninstall => cmd_uninstall::cmd_uninstall(cli.dry_run),
        Command::Status => status::cmd_status(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{kernels::Kernel, mkconfig::build_config};
    use std::path::PathBuf;

    const ESP: &str = "/boot";
    const CMDLINE: &str = "root=UUID=1234 rw quiet";

    fn fake_kernels() -> Vec<Kernel> {
        vec![
            Kernel {
                variant: "linux".to_string(),
                image: PathBuf::from("/boot/vmlinuz-linux"),
                initrd: Some(PathBuf::from("/boot/initramfs-linux.img")),
                fallback: Some(PathBuf::from("/boot/initramfs-linux-fallback.img")),
            },
            Kernel {
                variant: "linux-lts".to_string(),
                image: PathBuf::from("/boot/vmlinuz-linux-lts"),
                initrd: Some(PathBuf::from("/boot/initramfs-linux-lts.img")),
                fallback: None,
            },
        ]
    }

    fn fake_ukis() -> Vec<PathBuf> {
        vec![PathBuf::from("/boot/EFI/Linux/arch-linux.efi")]
    }

    fn fake_windows() -> Option<PathBuf> {
        Some(PathBuf::from("/boot/EFI/Microsoft/Boot/bootmgfw.efi"))
    }

    #[test]

    pub fn quantity_and_order() {
        let esp = PathBuf::from(ESP);
        let config = build_config(&fake_kernels(), &fake_ukis(), fake_windows(), CMDLINE, &esp);
        let expected = [
            "(UKI: arch-linux)",
            "(linux)",
            "(linux, fallback)",
            "(linux-lts)",
            "Windows Boot Manager",
        ];
        assert_eq!(config.entries.len(), expected.len());

        for (expected_name, entry) in expected.iter().zip(config.entries) {
            assert!(entry.name.ends_with(expected_name));
        }
    }

    #[test]

    pub fn uefi_paths() {
        let esp = PathBuf::from(ESP);
        let config = build_config(&fake_kernels(), &fake_ukis(), fake_windows(), CMDLINE, &esp);
        let entries = config.entries;
        assert_eq!(entries.len(), 5);
        assert_eq!(
            entries[0].path.as_deref(),
            Some(r"\EFI\Linux\arch-linux.efi")
        );
        assert_eq!(entries[0].kernel, None);
        assert_eq!(entries[1].kernel.as_deref(), Some(r"\vmlinuz-linux"));
        assert_eq!(entries[1].initrd.as_deref(), Some(r"\initramfs-linux.img"));
        assert_eq!(entries[1].cmdline.as_deref(), Some(CMDLINE));
        assert_eq!(
            entries[2].initrd.as_deref(),
            Some(r"\initramfs-linux-fallback.img")
        );
        assert_eq!(
            entries[4].path.as_deref(),
            Some(r"\EFI\Microsoft\Boot\bootmgfw.efi")
        );
        assert_eq!(entries[4].kernel, None);
    }

    #[test]

    pub fn no_fallback_when_missing() {
        let esp = PathBuf::from(ESP);
        let config = build_config(&fake_kernels()[1..], &[], None, CMDLINE, &esp);
        assert_eq!(config.entries.len(), 1);
        assert!(config.entries[0].name.ends_with("(linux-lts)"));

        for entry in config.entries {
            assert!(!entry.name.contains("fallback"));
        }
    }

    #[test]

    pub fn file_outside_esp_skipped() {
        let esp = PathBuf::from(ESP);
        let uki_path = PathBuf::from("/home/user/x.efi");
        let config = build_config(&[], &vec![uki_path], None, CMDLINE, &esp);
        assert_eq!(config.entries.len(), 0);
    }

    #[test]

    pub fn no_windows() {
        let esp = PathBuf::from(ESP);
        let config = build_config(&fake_kernels(), &fake_ukis(), None, CMDLINE, &esp);
        assert_eq!(config.entries.len(), 4);
        let win_search = config
            .entries
            .iter()
            .any(|entry| entry.name.ends_with("Windows Boot Manager"));

        assert_eq!(win_search, false);
    }

    #[test]

    pub fn config_roundtrip() {
        let esp = PathBuf::from(ESP);
        let config = build_config(&fake_kernels(), &fake_ukis(), fake_windows(), CMDLINE, &esp);
        let config_str = config.to_string();
        let config2 = config::parse(&config_str);
        assert!(config2.is_ok());
        assert_eq!(config, config2.unwrap());
    }
}
