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
    callback(99);
    Ok(())
}
