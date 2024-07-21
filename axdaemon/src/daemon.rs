use std::net::IpAddr;
use std::net::SocketAddr;

use colored::Colorize;
use futures_concurrency::stream::Merge;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};

use axdaemon_request::DaemonReply;
use axerrno::{ax_err_type, AxResult};

use crate::listener::VMMEventWrapper;
use crate::vmm::VMM;

/// Events to be handled.
/// * `VMM`: requests from axcli.
/// * `VDEV`: irqs from guest VM for emulated device operations (through UIO).
/// * `CtrlC`: Ctrl+C from os to terminate axdaemon process.
#[derive(Debug)]
pub enum Event {
    VMM(VMMEventWrapper),
    // VDEV,
    CtrlC,
}

#[derive(Debug, Default)]
struct Daemon {
    vmm: VMM,
}

impl Daemon {
    const fn init() -> Self {
        Self { vmm: VMM::new() }
    }

    pub async fn run(&mut self, bind: SocketAddr) -> AxResult {
        info!("{} running, bind to {}", "AxDaemon".bold().green(), bind);

        // Setup ctrlc events.
        let ctrlc_events = set_up_ctrlc_handler()?;

        // Setup VMM events, which comes from TCP connection.
        let (events_tx, events_rx) = flume::bounded(10);
        let _listen_port = crate::listener::spawn_listener_loop(bind, events_tx).await?;
        let vmm_events = events_rx.into_stream().map(|e| Event::VMM(e));

        let mut events = (ctrlc_events, vmm_events).merge();

        while let Some(event) = events.next().await {
            match event {
                Event::VMM(vmm_event) => self.handle_vmm_event(vmm_event).await?,
                // Event::VDEV => todo!(),
                Event::CtrlC => todo!(),
            }
        }
        Ok(())
    }

    async fn handle_vmm_event(&mut self, event: VMMEventWrapper) -> AxResult {
        match event {
            VMMEventWrapper { request, reply_tx } => {
                let result = self.vmm.handle_daemon_request(request);
                let reply = DaemonReply::Result(result.map_err(|err| err.to_string()));
                let _ = reply_tx.send(Some(reply)).map_err(|_| {
                    error!("could not send node info reply from daemon to coordinator")
                });
                Ok(())
            }
        }
    }
}

pub fn run(listen_addr: IpAddr, listen_port: u16) -> AxResult {
    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| ax_err_type!(BadState, format!("tokio runtime failed, {err:?}")))?;
    rt.block_on(async {
        let bind = SocketAddr::new(listen_addr, listen_port);
        let mut daemon = Daemon::init();
        daemon.run(bind).await
    })
}

fn set_up_ctrlc_handler() -> AxResult<impl Stream<Item = Event>> {
    let (ctrlc_tx, ctrlc_rx) = mpsc::channel(1);

    let mut ctrlc_sent = false;
    ctrlc::set_handler(move || {
        if ctrlc_sent {
            warn!("received second ctrlc signal -> aborting immediately");
            std::process::abort();
        } else {
            info!("received ctrlc signal");
            if ctrlc_tx.blocking_send(Event::CtrlC).is_err() {
                error!("failed to report ctrl-c event to dora-coordinator");
            }

            ctrlc_sent = true;
        }
    })
    .map_err(|err| ax_err_type!(BadState, format!("failed to set ctrl-c handler, {err:?}")))?;

    Ok(ReceiverStream::new(ctrlc_rx))
}
