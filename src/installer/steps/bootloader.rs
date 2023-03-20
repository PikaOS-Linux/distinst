use crate::chroot::Chroot;
use crate::disks::{Bootloader, Disks};
use crate::errors::IoContext;
use libc;
use os_release::OsRelease;
use std::{
    ffi::{OsStr, OsString},
    fs, io,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};
use crate::Config;
use crate::MODIFY_BOOT_ORDER;

use super::mount_efivars;

pub fn bootloader<F: FnMut(i32)>(
    disks: &Disks,
    mount_dir: &Path,
    bootloader: Bootloader,
    config: &Config,
    iso_os_release: &OsRelease,
    mut callback: F,
) -> io::Result<()> {
    // Obtain the root device & partition, with an optional EFI device & partition.
    let ((root_dev, _root_part), boot_opt) = disks.get_base_partitions(bootloader);

    let mut efi_part_num = 0;

    let bootloader_dev = boot_opt.map_or(root_dev, |(dev, dev_part)| {
        efi_part_num = dev_part.number;
        dev
    });

    info!("{}: installing bootloader for {:?}", bootloader_dev.display(), bootloader);

    {
            let mut chroot = Chroot::new(mount_dir)?;
            let efivars_mount = mount_efivars(&mount_dir)?;

            match bootloader {
                Bootloader::Bios => {
                    chroot
                        .command(
                            "grub-install",
                            &[
                                // Recreate device map
                                "--recheck".into(),
                                // Install for BIOS
                                "--target=i386-pc".into(),
                                // Install to the bootloader_dev device
                                bootloader_dev.to_str().unwrap().to_owned(),
                            ],
                        )
                        .run()?;

                    chroot.command("update-initramfs", &["-c", "-k", "all"]).run()?;
                }
                Bootloader::Efi => {
                    // Grub disallows whitespaces in the name.
                    let name = super::normalize_os_release_name(&iso_os_release.name);
                    if &name == "Pop!_OS" {
                        chroot
                            .command(
                                "bootctl",
                                &[
                                    // Install systemd-boot
                                    "install",
                                    // Provide path to ESP
                                    "--path=/boot/efi",
                                    // Do not set EFI variables
                                    "--no-variables",
                             ],
                        )
                        .run()?;

                    chroot.command("update-initramfs", &["-c", "-k", "all"]).run()?;
                }
                }
            }

            // Sync to the disk before unmounting
            unsafe {
                libc::sync();
            }

            drop(efivars_mount);
            chroot.unmount(false)?;
        }
    }

    callback(99);

    Ok(())
}
