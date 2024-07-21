use std::path::PathBuf;
use std::net::{IpAddr, Ipv4Addr};

pub const ARCEOS_DAEMON_PORT_DEFAULT: u16 = 2334;

pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum DaemonRequest {
    RegisterVM { vmid: usize, disk_image_path: PathBuf },
    BootVM { vmid: usize },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[must_use]
pub enum DaemonReply {
    Result(Result<(), String>),
    Empty,
}