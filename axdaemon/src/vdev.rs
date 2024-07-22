use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use colored::Colorize;
use memmap::{MmapMut, MmapOptions};

use axerrno::{ax_err, ax_err_type, AxResult};

/// Currently 2MB.
/// I get a warning from kernel output when I try to set page size for HugeTLB as 32MB.
///      `HugeTLB: unsupported default_hugepagesz 33554432. Reverting to 2097152`
/// Todo: make it 32MB through some way.
const HUGE_TLB_MAX: usize = 2 * 1024 * 1024;

const BLOCK_SIZE: usize = 512;

pub struct BlockRequest {
    req_type: usize,
    sector: usize,
    count: usize,
}

/// Events related to emulated device, e.g. Virtio-Blk request.
/// See crate `xxx` for details.
/// * `request`: emulated device request from guest VM.
/// * `reply_tx`: returned `notify` refers to request that have been processed.
#[derive(Debug)]
pub struct VDevEventWrapper {
    // pub request: DaemonRequest,
    // pub reply_tx: oneshot::Sender<Option<DaemonReply>>,
}

#[derive(Debug)]
struct EmulatedBlock {
    base: EmulatedBlockCfgMmio,
    drive_file: DriveFile,
}

#[repr(C)]
#[derive(Debug, Default)]
struct EmulatedBlockCfgMmio {
    vmid: usize,
    block_num: usize,
    dma_block_max: usize,
    cache_size: usize,
    cache_gva: usize,
    cache_gpa: usize,
    // This field was write by hypervisor.
    cache_hpa: usize,
}

/// Represent a single drive backend file.
#[derive(Debug)]
pub struct DriveFile {
    /// VM id.
    vmid: usize,
    /// The opened file.
    file: File,
    /// File path.
    path: PathBuf,
}

#[derive(Debug, Default)]
pub struct EmulatedBlockBackends {
    emulated_blocks: HashMap<usize, EmulatedBlock>,
}

impl EmulatedBlockBackends {
    /// Add a drive file for target VM.
    pub fn setup_emulated_block(&mut self, vmid: usize, path: PathBuf, direct: bool) -> AxResult {
        info!(
            "{} set up emulated block {:?} for VM [{}]",
            "AxDaemon".bold().green(),
            path,
            vmid
        );

        let file = open_file(&path, direct)?;
        let file_size = file.metadata().unwrap().len();
        let drive_file = DriveFile { vmid, file, path };

        let mut base = EmulatedBlockCfgMmio::default();
        setup_emulated_block_rw_cache(&mut base, file_size)?;

        let emulated_block = EmulatedBlock { base, drive_file };

        self.emulated_blocks.insert(vmid, emulated_block);
        Ok(())
    }

    /// Remove a drive file according to vmid.
    pub fn remove_emulated_block(&mut self, vmid: usize) -> AxResult {
        let _removed_block = self.emulated_blocks.remove(&vmid).ok_or(ax_err_type!(
            InvalidInput,
            format!(
                "Failed to remove drive file for VM[{}], it does not exist",
                vmid
            )
        ))?;

        // Todo: release sources allocated for `EmulatedBlock` here.

        Ok(())
    }

    pub fn emulated_block_rw_sectors(&mut self, vmid: usize, req: BlockRequest) -> AxResult {
        let block = self.emulated_blocks.get_mut(&vmid).ok_or(ax_err_type!(
            InvalidInput,
            format!("VM[]'s emulated block not exists")
        ))?;
        block.rw_sectors(req)
    }
}

impl EmulatedBlock {
    fn rw_sectors(&mut self, _req: BlockRequest) -> AxResult {
        // self.drive_file.
        Ok(())
    }
}

pub fn open_file(path: &PathBuf, direct: bool) -> AxResult<File> {
    let mut options = OpenOptions::new();
    // READ/WRITE by default, may add READ_ONLY in the future?
    options.read(true).write(true);
    if direct {
        options.custom_flags(libc::O_DIRECT);
    }
    let file = options.open(path).map_err(|err| {
        ax_err_type!(
            InvalidInput,
            format!(
                "failed to open the file for block {:?}. Error:{:?}\nos err :{}",
                path,
                err,
                std::io::Error::last_os_error(),
            )
        )
    })?;

    Ok(file)
}

fn setup_emulated_block_rw_cache(
    base: &mut EmulatedBlockCfgMmio,
    drive_file_size: u64,
) -> AxResult {
    let cache_size = HUGE_TLB_MAX;

    // // system("mkdir -p /mnt/huge");
    // let _ = std::process::Command::new("mkdir")
    //     .arg("-p")
    //     .arg("/mnt/huge")
    //     .output()
    //     .expect("failed to execute process");
    // // system("mount -t hugetlbfs -o pagesize=2M none /mnt/huge");
    // let _ = std::process::Command::new("mount")
    //     .arg("-t")
    //     .arg("hugetlbfs")
    //     .arg("-o")
    //     .arg("pagesize=2M")
    //     .arg("none")
    //     .arg("/mnt/huge")
    //     .output()
    //     .expect("failed to execute process");

    let mut mmap_options = MmapOptions::new();
    let mut mmap: MmapMut = mmap_options
        // .huge_tlb()
        .len(cache_size)
        .map_anon()
        .map_err(|err| ax_err_type!(BadState, format! {"failed to mmap {err:?}"}))?;

    check_cache_address(&mut mmap)?;

    base.block_num = (drive_file_size as f64 / BLOCK_SIZE as f64).ceil() as usize;
    base.cache_size = cache_size;
    base.dma_block_max = cache_size / BLOCK_SIZE;
    base.cache_gva = mmap.as_ptr() as usize;
    base.cache_gpa = get_physical_addr(&mmap)?;
    // This field was set by hypervisor
    base.cache_hpa = 0xdead_beef;

    // Notify hypervisor through ioctl & hvc.

    Ok(())
}

/// Check if cache address allocated by mmap is valid.
fn check_cache_address(mmap: &mut MmapMut) -> AxResult {
    extern crate rand;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    // Gererate random bytes for test.
    let rand_bytes: Vec<u8> = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(HUGE_TLB_MAX)
        .collect();

    // Test write.
    mmap.copy_from_slice(rand_bytes.as_slice());

    // Test read.
    let mut tmp = [0 as u8; HUGE_TLB_MAX];
    tmp.copy_from_slice(mmap.as_ref());

    // Judge equal.
    if !tmp.eq(rand_bytes.as_slice()) {
        return ax_err!(
            BadState,
            format!("Cache of {} Bytes for is invalid", HUGE_TLB_MAX)
        );
    }

    Ok(())
}

fn get_physical_addr(mmap: &MmapMut) -> AxResult<usize> {
    use libc::c_ulong;
    use std::io::{Read, Seek};

    let vaddr = mmap.as_ptr() as usize;

    let pagemap_path = format!("/proc/{}/pagemap", std::process::id());
    let mut file = File::open(&pagemap_path).map_err(|err| {
        ax_err_type!(InvalidInput, format!("Open {} err: {}", &pagemap_path, err))
    })?;

    let page_size: usize = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) } as usize;
    let offset = ((vaddr as usize) / page_size) * (core::mem::size_of::<c_ulong>() as usize);

    file.seek(std::io::SeekFrom::Start(offset as u64))
        .map_err(|err| {
            ax_err_type!(
                BadAddress,
                format!(
                    "seek on {} at offset {:x} err, {:?}",
                    pagemap_path.as_str(),
                    offset,
                    err
                )
            )
        })?;

    let mut pagemap_entry: c_ulong = 0;

    file.read_exact(unsafe {
        core::slice::from_raw_parts_mut(
            &mut pagemap_entry as *mut _ as *mut u8,
            core::mem::size_of::<c_ulong>(),
        )
    })
    .map_err(|err| ax_err_type!(BadAddress, format!("Read page table entry err {:?}", err)))?;

    // Note:
    // entry->soft_dirty = (data >> 55) & 1;
    // entry->file_page = (data >> 61) & 1;
    // entry->swapped = (data >> 62) & 1;
    // entry->present = (data >> 63) & 1;

    if (pagemap_entry & (1 << 63)) == 0 {
        return ax_err!(
            BadAddress,
            format!(
                "Virtual Address {:#x} converts to paddr err: page not in memory",
                vaddr
            )
        );
    }

    let pfn = pagemap_entry & ((1 << 55) - 1);
    let paddr = pfn as usize * page_size + vaddr % page_size;
    Ok(paddr)
}
