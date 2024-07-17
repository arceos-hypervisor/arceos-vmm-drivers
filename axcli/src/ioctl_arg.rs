use core::ffi::c_void;
use rustix::ioctl;

#[derive(Debug, Default)]
#[repr(C)]
pub struct VmCreateIoctlArg {
    pub id: usize,
    // pub vm_name_ptr: usize,
    // pub vm_name_len: usize,
    // pub vm_type: usize,
    pub cpu_set: usize,
    // pub bios_load_addr: usize,
    pub bios_img_ptr: usize,
    pub bios_img_size: usize,
    // pub kernel_load_addr: usize,
    pub kernel_img_ptr: usize,
    pub kernel_img_size: usize,
    // pub ramdisk_load_addr: usize,
    pub ramdisk_img_ptr: usize,
    pub ramdisk_img_size: usize,

    pub raw_cfg_file_ptr: usize,
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