use core::ffi::c_void;
use rustix::ioctl;

#[derive(Debug, Default)]
#[repr(C)]
pub struct VmCreateIoctlArg {
    /// VM id.
    pub id: usize,
    /// VM cpu mask.
    pub cpu_set: usize,
    /// User address of bios image file.
    pub bios_img_ptr: usize,
    /// Size of bios image file.
    pub bios_img_size: usize,
    /// User address of kernel image file.
    pub kernel_img_ptr: usize,
    /// Size of kernel image file.
    pub kernel_img_size: usize,
    /// User address of ramdisk image file (Default to 0 if no ramdisk is needed).
    pub ramdisk_img_ptr: usize,
    /// Size of ramdisk image file (Default to 0 if no ramdisk is needed).
    pub ramdisk_img_size: usize,

    /// User address which stores the disk image path.
    pub disk_image_path_ptr: usize,
    /// Length of disk image path.
    pub disk_image_path_length: usize,

    /// User address which stores the raw TOML config file in String format.
    pub raw_cfg_file_ptr: usize,
    /// Size of the raw TOML config file in String format.
    pub raw_cfg_file_size: usize,
}

unsafe impl ioctl::Ioctl for VmCreateIoctlArg {
    type Output = ();

    const OPCODE: ioctl::Opcode = ioctl::Opcode::write::<Self>(0, 6);

    const IS_MUTATING: bool = false;

    fn as_ptr(&mut self) -> *mut c_void {
        self as *const _ as *mut c_void
    }

    unsafe fn output_from_ptr(
        _out: ioctl::IoctlOutput,
        _extract_output: *mut c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct VmBootIoctlArg {
    pub id: usize,
}

unsafe impl ioctl::Ioctl for VmBootIoctlArg {
    type Output = ();

    const OPCODE: ioctl::Opcode = ioctl::Opcode::write::<Self>(0, 7);

    const IS_MUTATING: bool = false;

    fn as_ptr(&mut self) -> *mut c_void {
        self as *const _ as *mut c_void
    }

    unsafe fn output_from_ptr(
        _out: ioctl::IoctlOutput,
        _extract_output: *mut c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct VmShutdownIoctlArg {
    pub id: usize,
}

unsafe impl ioctl::Ioctl for VmShutdownIoctlArg {
    type Output = ();

    const OPCODE: ioctl::Opcode = ioctl::Opcode::write::<Self>(0, 8);

    const IS_MUTATING: bool = false;

    fn as_ptr(&mut self) -> *mut c_void {
        self as *const _ as *mut c_void
    }

    unsafe fn output_from_ptr(
        _out: ioctl::IoctlOutput,
        _extract_output: *mut c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct VmGetDiskImgPathIoctlArg {
    pub id: usize,
    pub disk_image_path_ptr: usize,
}

unsafe impl ioctl::Ioctl for VmGetDiskImgPathIoctlArg {
    type Output = ();

    const OPCODE: ioctl::Opcode = ioctl::Opcode::read::<Self>(0, 9);

    const IS_MUTATING: bool = false;

    fn as_ptr(&mut self) -> *mut c_void {
        self as *const _ as *mut c_void
    }

    unsafe fn output_from_ptr(
        _out: ioctl::IoctlOutput,
        _extract_output: *mut c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}
