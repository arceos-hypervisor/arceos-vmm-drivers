use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VmCreateCliArg {
    // Basic Information
    pub id: usize,
    pub name: String,
    pub vm_type: usize,
    // Resources.
    pub cpu_set: usize,

    pub entry_point: usize,

    pub bios_path: String,
    pub bios_load_addr: usize,

    pub kernel_path: String,
    pub kernel_load_addr: usize,

    pub ramdisk_path: Option<String>,
    pub ramdisk_load_addr: Option<usize>,

    pub disk_path: Option<String>,

    /// Memory Information
    memory_regions: Vec<VmMemCfg>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct VmMemCfg {
    pub gpa: usize,
    pub size: usize,
    pub flags: usize,
}
