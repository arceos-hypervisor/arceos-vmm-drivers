//! A daemon process for guest VMs in ArceOS Hypervisor.

use axerrno::AxResult;
use clap::Parser;
use colored::Colorize;
use std::net::{IpAddr, Ipv4Addr};

mod daemon;
mod listener;
mod tcp_utils;
mod vmm;

pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
// const LISTEN_WILDCARD: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

#[macro_use]
extern crate log;

#[derive(Debug, clap::Parser)]
#[clap(version)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

/// dora-rs cli client
#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Start daemon.
    Init {
        /// Address of the dora coordinator
        #[clap(long, value_name = "IP", default_value_t = LOCALHOST)]
        listen_addr: IpAddr,
        /// Port number of the coordinator control server
        #[clap(long, value_name = "PORT", default_value_t = axdaemon_request::ARCEOS_DAEMON_PORT_DEFAULT)]
        listen_port: u16,
        /// Run the daemon in background
        #[clap(long, action)]
        detach: bool,
    },
}

fn main() {
    // configure logger and set log level
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    if let Err(err) = run() {
        eprintln!("\n\n{}", "[ERROR]".bold().red());
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> AxResult {
    let args = Args::parse();

    match args.command {
        Command::Init {
            listen_addr,
            listen_port,
            detach: _,
        } => {
            daemon::run(listen_addr, listen_port)?;
        }
    }

    Ok(())
}
