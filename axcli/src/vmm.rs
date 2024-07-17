use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;

use rustix::fd::OwnedFd;
use rustix::fs::{open, Mode, OFlags};
use rustix::ioctl;

use crate::cfg::VmCreateCliArg;
use crate::cli::{VmBootShutdownArgs, VmCreateArgs};
use crate::ioctl_arg::{VmBootIoctlArg, VmCreateIoctlArg, VmShutdownIoctlArg};

pub fn open_driver() -> OwnedFd {
    let driver = String::from("/dev/jailhouse");
    open(driver, OFlags::RDWR, Mode::RWXO).expect("Failed to open ArceOS driver")
}

pub fn perform_ioctl<I: ioctl::Ioctl>(ioctl: I) {
    let fd = open_driver();
    unsafe { ioctl::ioctl(fd, ioctl) }.expect("failed to perform ioctl");
}

pub fn axvmm_create_vm(arg: VmCreateArgs) -> Result<(), String> {
    let config_content = read_to_string(arg.config_path).map_err(|err| err.to_string())?;

    let vm_arg: VmCreateCliArg =
        toml::from_str(config_content.as_str()).map_err(|err| err.to_string())?;

    debug!("get vm_arg {:#x?}", vm_arg);

    let mut bios_img = File::open(vm_arg.bios_path).unwrap();
    let mut kernel_img = File::open(vm_arg.kernel_path).unwrap();

    let bios_img_size = bios_img.metadata().unwrap().len() as usize;
    let kernel_img_size = kernel_img.metadata().unwrap().len() as usize;

    let mut bios_img_buffer = Vec::new();
    bios_img.read_to_end(&mut bios_img_buffer).unwrap();

    let mut kernel_img_buffer = Vec::new();
    kernel_img.read_to_end(&mut kernel_img_buffer).unwrap();

    let mut ramdisk_img_ptr = 0;
    let mut ramdisk_img_size = 0;
    let mut ramdisk_img_buffer = Vec::new();

    let _ramdisk_img = vm_arg.ramdisk_path.map(|ramdisk_path| {
        let mut ramdisk_file = File::open(ramdisk_path).unwrap();
        ramdisk_img_size = ramdisk_file.metadata().unwrap().len() as usize;
        ramdisk_file.read_to_end(&mut ramdisk_img_buffer).unwrap();
        ramdisk_img_ptr = ramdisk_img_buffer.as_ptr() as usize;
        ramdisk_file
    });

    let driver_arg = VmCreateIoctlArg {
        id: vm_arg.id,
        // vm_name_ptr: vm_arg.name.as_ptr() as usize,
        // vm_name_len: vm_arg.name.len(),
        // vm_type: vm_arg.vm_type,
        cpu_set: vm_arg.cpu_set,
        // bios_load_addr: vm_arg.bios_load_addr,
        bios_img_ptr: bios_img_buffer.as_ptr() as usize,
        bios_img_size,
        // kernel_load_addr: vm_arg.kernel_load_addr,
        kernel_img_ptr: kernel_img_buffer.as_ptr() as usize,
        kernel_img_size,
        // ramdisk_load_addr: match vm_arg.ramdisk_load_addr {
        //     Some(ramdisk_load_addr) => ramdisk_load_addr,
        //     None => 0,
        // },
        ramdisk_img_ptr,
        ramdisk_img_size,
        raw_cfg_file_ptr: config_content.as_ptr() as usize,
        raw_cfg_file_size: config_content.len(),
    };

    perform_ioctl(driver_arg);

    Ok(())
}

pub fn axvmm_boot_shutdown_vm(is_boot: bool, arg: VmBootShutdownArgs) -> Result<(), String> {
    let vmid = arg.vmid;
    if is_boot {
        println!("Boot VM [{}]", vmid);
        let driver_arg = VmBootIoctlArg { id: vmid as usize };
        perform_ioctl(driver_arg);
    } else {
        println!("Shutdown VM [{}]", vmid);
        let driver_arg = VmShutdownIoctlArg { id: vmid as usize };
        perform_ioctl(driver_arg);
    }

    Ok(())
}
