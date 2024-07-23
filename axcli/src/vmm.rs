use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;

use colored::Colorize;
use rustix::fd::OwnedFd;
use rustix::fs::{open, Mode, OFlags};
use rustix::ioctl;

use axerrno::{AxError, AxResult};

use crate::cfg::VmCreateCliArg;
use crate::cli::{VmCreateArgs, VmIdArgs};
use crate::ioctl_arg::{VmBootIoctlArg, VmCreateIoctlArg, VmShutdownIoctlArg};

pub fn open_driver() -> OwnedFd {
    let driver = String::from("/dev/jailhouse");
    open(driver, OFlags::RDWR, Mode::RWXO).expect("Failed to open ArceOS driver")
}

pub fn perform_ioctl<I: ioctl::Ioctl>(ioctl: I) {
    let fd = open_driver();
    unsafe { ioctl::ioctl(fd, ioctl) }.expect("failed to perform ioctl");
}

pub fn axvmm_create_vm(arg: VmCreateArgs) -> AxResult {
    let config_content = read_to_string(arg.config_path).map_err(|err| {
        warn!("Failed to get VM config file {err:?}");
        AxError::InvalidInput
    })?;

    let vm_arg: VmCreateCliArg = toml::from_str(config_content.as_str()).map_err(|err| {
        warn!("Failed to deserialize VM config file {err:?}");
        AxError::InvalidInput
    })?;

    debug!("get vm_arg {:#x?}", vm_arg);

    let mut bios_img = File::open(vm_arg.bios_path.clone()).map_err(|err| {
        warn!(
            "Failed to open bios file on {:?}, {:?}",
            vm_arg.bios_path, err
        );
        AxError::InvalidInput
    })?;
    let mut kernel_img = File::open(vm_arg.kernel_path.clone()).map_err(|err| {
        warn!(
            "Failed to open kernel file on {:?}, {:?}",
            vm_arg.kernel_path, err
        );
        AxError::InvalidInput
    })?;

    let bios_img_size = bios_img.metadata().unwrap().len() as usize;
    let kernel_img_size = kernel_img.metadata().unwrap().len() as usize;

    let mut bios_img_buffer = Vec::new();
    bios_img
        .read_to_end(&mut bios_img_buffer)
        .expect("Failed to read bios image file");

    let mut kernel_img_buffer = Vec::new();
    kernel_img
        .read_to_end(&mut kernel_img_buffer)
        .expect("Failed to read kernel image file");

    let mut ramdisk_img_ptr = 0;
    let mut ramdisk_img_size = 0;
    let mut ramdisk_img_buffer = Vec::new();

    let _ramdisk_img = vm_arg.ramdisk_path.map(|ramdisk_path| {
        let mut ramdisk_file = File::open(ramdisk_path.clone())
            .map_err(|err| {
                warn!(
                    "Failed to open ramdisk file on {:?}, {:?}",
                    ramdisk_path, err
                );
                AxError::InvalidInput
            })
            .unwrap();
        ramdisk_img_size = ramdisk_file.metadata().unwrap().len() as usize;
        ramdisk_file
            .read_to_end(&mut ramdisk_img_buffer)
            .expect("Failed to read ramdisk image file");
        ramdisk_img_ptr = ramdisk_img_buffer.as_ptr() as usize;
        ramdisk_file
    });

    // VM id could be modified by hypervisor.
    let vmid = vm_arg.id;

    let driver_arg = VmCreateIoctlArg {
        id_ptr: &vmid as *const _ as usize,
        cpu_set: vm_arg.cpu_set,
        bios_img_ptr: bios_img_buffer.as_ptr() as usize,
        bios_img_size,
        kernel_img_ptr: kernel_img_buffer.as_ptr() as usize,
        kernel_img_size,
        ramdisk_img_ptr,
        ramdisk_img_size,
        raw_cfg_file_ptr: config_content.as_ptr() as usize,
        raw_cfg_file_size: config_content.len(),
    };

    perform_ioctl(driver_arg);

    info!(
        "VM [{vmid}] created success! Trying to register to {} ...",
        "AxDaemon".bold().green()
    );

    if let Some(path_str) = vm_arg.disk_path {
        crate::daemon::register_vm_to_daemon(vmid, std::path::PathBuf::from(path_str));
    }

    Ok(())
}

pub fn axvmm_boot_vm(arg: VmIdArgs) -> Result<(), String> {
    let id = arg.vmid as usize;

    crate::daemon::setup_vm_on_daemon(id);

    println!("Boot VM [{}]", id);
    let driver_arg = VmBootIoctlArg { id };
    perform_ioctl(driver_arg);

    Ok(())
}

pub fn axvmm_shutdown_vm(arg: VmIdArgs) -> Result<(), String> {
    let id = arg.vmid as usize;
    println!("Shutdown VM [{}]", id);
    let driver_arg = VmShutdownIoctlArg { id };
    perform_ioctl(driver_arg);
    Ok(())
}
