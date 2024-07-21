use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use colored::Colorize;

use axdaemon_request::DaemonRequest;
use axerrno::{ax_err, ax_err_type, AxResult};

#[derive(Debug, Default)]
pub struct VMM {
    vm_disk_image_paths: Mutex<BTreeMap<usize, PathBuf>>,
}

impl VMM {
    pub const fn new() -> Self {
        Self {
            vm_disk_image_paths: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn handle_daemon_request(&self, request: DaemonRequest) -> AxResult {
        match request {
            axdaemon_request::DaemonRequest::RegisterVM {
                vmid,
                disk_image_path,
            } => self.add_vm_disk_images(vmid, disk_image_path)?,
            axdaemon_request::DaemonRequest::BootVM { vmid: _ } => todo!(),
        }
        Ok(())
    }
}

impl VMM {
    fn add_vm_disk_images(&self, vmid: usize, image_path: PathBuf) -> AxResult {
        if self
            .vm_disk_image_paths
            .lock()
            .map_err(|err| ax_err_type!(BadState, format!("failed to get lock {err:?}")))?
            .contains_key(&vmid)
        {
            return ax_err!(
                InvalidInput,
                format!("VM [{vmid}] has already been registered in AxDaemon")
            );
        }

        info!(
            "{} register VM {} disk_path {:?}",
            "AxDaemon".bold().green(),
            vmid,
            image_path
        );

        self.vm_disk_image_paths
            .lock()
            .unwrap()
            .insert(vmid, image_path);
        Ok(())
    }

    pub fn get_vm_disk_images(&self, vmid: usize) -> Option<PathBuf> {
        self.vm_disk_image_paths.lock().unwrap().get(&vmid).cloned()
    }
}
