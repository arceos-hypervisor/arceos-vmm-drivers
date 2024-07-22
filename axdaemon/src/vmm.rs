use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use colored::Colorize;
use tokio::sync::oneshot;

use axdaemon_request::{DaemonReply, DaemonRequest};
use axerrno::{ax_err, ax_err_type, AxError, AxResult};

use crate::vdev::EmulatedBlockBackends;

/// Events related to VM management, e.g. VM register, boot, shutdown, remove.
/// See crate `axdaemon_request` for details.
/// * `request`: `DaemonRequest` sent by axcli through IPC (local socket for now).
/// * `reply_tx`: returned `DaemonReply` informs processing result.
#[derive(Debug)]
pub struct VMMEventWrapper {
    pub request: DaemonRequest,
    pub reply_tx: oneshot::Sender<Option<DaemonReply>>,
}

#[derive(Debug, Default)]
pub struct VMM {
    vm_disk_image_paths: Mutex<BTreeMap<usize, PathBuf>>,
    vdevs: EmulatedBlockBackends,
}

impl VMM {
    pub fn new() -> Self {
        Self {
            vm_disk_image_paths: Mutex::new(BTreeMap::new()),
            ..Default::default()
        }
    }

    pub fn handle_daemon_request(&mut self, request: DaemonRequest) -> AxResult {
        match request {
            axdaemon_request::DaemonRequest::RegisterVM {
                vmid,
                disk_image_path,
            } => self.add_vm_disk_images(vmid, disk_image_path)?,
            axdaemon_request::DaemonRequest::BootVM { vmid } => self.setup_vm(vmid)?,
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

    fn get_vm_disk_image(&self, vmid: usize) -> Option<PathBuf> {
        self.vm_disk_image_paths.lock().unwrap().get(&vmid).cloned()
    }

    pub fn setup_vm(&mut self, vmid: usize) -> AxResult {
        info!("{} set up VM [{}]", "AxDaemon".bold().green(), vmid);

        let disk_image_path = self.get_vm_disk_image(vmid).ok_or(AxError::BadState)?;

        self.vdevs
            .setup_emulated_block(vmid, disk_image_path, true)?;

        info!(
            "{} set up VM [{}] success, it is ready for booting...",
            "AxDaemon".bold().green(),
            vmid
        );
        Ok(())
    }
}
