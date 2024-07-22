use std::io::ErrorKind;
use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

use axdaemon_request::DaemonRequest;
use axerrno::{ax_err, ax_err_type, AxResult};

use crate::tcp_utils::{tcp_receive, tcp_send};
use crate::vmm::VMMEventWrapper;

pub async fn spawn_listener_loop(
    bind: SocketAddr,
    events_tx: flume::Sender<VMMEventWrapper>,
) -> AxResult<u16> {
    let socket = match TcpListener::bind(bind).await {
        Ok(socket) => socket,
        Err(err) => {
            warn!("TcpListen bind err {:?}", err);
            return ax_err!(BadState, "failed to create local TCP listener");
        }
    };

    let listen_port = socket
        .local_addr()
        .map_err(|err| {
            warn!("TcpListen local_addr err {:?}", err);
            ax_err_type!(BadState, "failed to get socket local_addr")
        })?
        .port();

    tokio::spawn(async move {
        listener_loop(socket, events_tx).await;
        debug!("Local listener loop finished");
    });

    Ok(listen_port)
}

async fn listener_loop(listener: TcpListener, events_tx: flume::Sender<VMMEventWrapper>) {
    loop {
        match listener.accept().await {
            Err(err) => {
                warn!("TcpListen accept err {:?}", err);
            }
            Ok((connection, _)) => {
                tokio::spawn(handle_connection_loop(connection, events_tx.clone()));
            }
        }
    }
}

async fn handle_connection_loop(
    mut connection: TcpStream,
    events_tx: flume::Sender<VMMEventWrapper>,
) {
    if let Err(err) = connection.set_nodelay(true) {
        warn!("failed to set nodelay for connection: {err}");
    }

    loop {
        match receive_message(&mut connection).await {
            Ok(Some(daemon_request)) => {
                let (reply_tx, reply_rx) = oneshot::channel();
                if events_tx
                    .send_async(VMMEventWrapper {
                        request: daemon_request,
                        reply_tx,
                    })
                    .await
                    .is_err()
                {
                    break;
                }
                let Ok(reply) = reply_rx.await else {
                    warn!("daemon sent no reply");
                    continue;
                };
                if let Some(reply) = reply {
                    let serialized = match bincode::serialize(&reply).map_err(|err| {
                        ax_err_type!(
                            InvalidData,
                            format!("failed to serialize DaemonReply {err:?}")
                        )
                    }) {
                        Ok(r) => r,
                        Err(err) => {
                            error!("{err:?}");
                            continue;
                        }
                    };

                    if let Err(err) = tcp_send(&mut connection, &serialized).await {
                        warn!("failed to send reply: {err}");
                        continue;
                    };
                }
                // crate::daemon::handle_request(daemon_request);
            }
            Ok(None) => break,
            Err(err) => {
                warn!("{err:?}");
                break;
            }
        }
    }
}

async fn receive_message(connection: &mut TcpStream) -> AxResult<Option<DaemonRequest>> {
    let raw = match tcp_receive(connection).await {
        Ok(raw) => raw,
        Err(err) => match err.kind() {
            ErrorKind::UnexpectedEof
            | ErrorKind::ConnectionAborted
            | ErrorKind::ConnectionReset => return Ok(None),
            _other => {
                warn!("tcp recive err {:?}", err);
                return ax_err!(
                    BadState,
                    "unexpected I/O error while trying to receive DaemonRequest"
                );
            }
        },
    };
    bincode::deserialize(&raw)
        .map_err(|err| {
            warn!("bincode::deserialize get err {:?}", err);
            ax_err_type!(InvalidData, "failed to deserialize DaemonRequest")
        })
        .map(Some)
}
