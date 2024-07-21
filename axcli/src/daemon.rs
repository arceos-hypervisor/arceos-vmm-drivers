use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;

use colored::Colorize;

use axdaemon_request::{DaemonReply, DaemonRequest, ARCEOS_DAEMON_PORT_DEFAULT, LOCALHOST};
use axerrno::{ax_err, ax_err_type, AxError, AxResult};

/// Register VM information to axdaemon process.
pub fn register_vm_to_daemon(vmid: usize, disk_image_path: PathBuf) {
    request_daemon(DaemonRequest::RegisterVM {
        vmid,
        disk_image_path,
    })
    .expect("Failed to register VM to axdaemon");
}

/// Setup VM on axdaemon process.
///
/// Including:
/// * Virtio-Blk service if needed.
pub fn setup_vm_on_daemon(vmid: usize) {
    request_daemon(DaemonRequest::BootVM { vmid }).expect("Failed to setup VM on axdaemon");
}

fn request_daemon(request: DaemonRequest) -> AxResult {
    let daemon_ip = match std::env::var("AXDAEMON_IP") {
        Ok(ip) => IpAddr::from_str(ip.as_str()).unwrap_or(LOCALHOST),
        Err(_) => LOCALHOST,
    };

    let daemon_port = match std::env::var("AXDAEMON_PORT") {
        Ok(port) => u16::from_str(port.as_str()).unwrap_or(ARCEOS_DAEMON_PORT_DEFAULT),
        Err(_) => ARCEOS_DAEMON_PORT_DEFAULT,
    };

    let daemon_addr = SocketAddr::new(daemon_ip, daemon_port);

    let mut stream = TcpStream::connect(daemon_addr).map_err(|err| {
        warn!(
            "[{}] failed to open TCP connection {err:?}\n{}",
            "AxCli".bold().purple(),
            format!(
                "Note: please check if {} is running as well as the ip address and port",
                "AxDaemon".bold().green()
            )
        );
        AxError::BadState
    })?;
    stream
        .set_nodelay(true)
        .map_err(|err| ax_err_type!(BadState, format!("failed to set nodelay {err:?}")))?;

    let message = bincode::serialize(&request).map_err(|err| {
        ax_err_type!(
            InvalidData,
            format!("failed to serialize DaemonRequest {err:?}")
        )
    })?;

    tcp_send(&mut stream, &message)
        .map_err(|err| ax_err_type!(BadState, format!("failed to send DaemonRequest {err:?}")))?;

    let reply = receive_reply(&mut stream).and_then(|reply| {
        reply.ok_or_else(|| ax_err_type!(BadState, "server disconnected unexpectedly"))
    })?;

    match reply {
        DaemonReply::Result(result) => result.map_err(|e| ax_err_type!(BadState, e.as_str()))?,
        other => warn!(
            "[{}] unexpected register reply: {other:?}",
            "AxCli".bold().purple()
        ),
    }

    Ok(())
}

fn receive_reply(connection: &mut TcpStream) -> AxResult<Option<DaemonReply>> {
    let raw = match tcp_receive(connection) {
        Ok(raw) => raw,
        Err(err) => match err.kind() {
            std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::ConnectionAborted => {
                return Ok(None)
            }
            other => {
                return ax_err!(
                    BadState,
                    format!(
                        "unexpected I/O error (kind {other:?}) while trying to receive DaemonReply"
                    )
                )
            }
        },
    };
    bincode::deserialize(&raw)
        .map_err(|err| {
            ax_err_type!(
                InvalidData,
                format!("failed to deserialize DaemonReply, {err:?}")
            )
        })
        .map(Some)
}

fn tcp_send(connection: &mut (impl Write + Unpin), message: &[u8]) -> std::io::Result<()> {
    let len_raw = (message.len() as u64).to_le_bytes();
    connection.write_all(&len_raw)?;
    connection.write_all(message)?;
    connection.flush()?;
    Ok(())
}

fn tcp_receive(connection: &mut (impl Read + Unpin)) -> std::io::Result<Vec<u8>> {
    let reply_len = {
        let mut raw = [0; 8];
        connection.read_exact(&mut raw)?;
        u64::from_le_bytes(raw) as usize
    };
    let mut reply = vec![0; reply_len];
    connection.read_exact(&mut reply)?;
    Ok(reply)
}
