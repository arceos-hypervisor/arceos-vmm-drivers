#[macro_use]
extern crate log;

mod cfg;
mod cli;
mod daemon;
mod ioctl_arg;
mod vmm;

use clap::Parser;
use cli::{CLISubCmd, HvSubCmd, VmSubCmd, CLI};

fn main() {
    // configure logger and set log level
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cli = CLI::parse();
    match cli.subcmd {
        CLISubCmd::Hv { subcmd } => match subcmd {
            HvSubCmd::Enable => todo!(),
            HvSubCmd::Disable => todo!(),
        },
        CLISubCmd::Vm { subcmd } => match subcmd {
            VmSubCmd::List => todo!(),
            VmSubCmd::Create(arg) => vmm::axvmm_create_vm(arg).expect("Failed to create VM"),
            VmSubCmd::Boot(arg) => vmm::axvmm_boot_vm(arg).expect("Failed to boot VM"),
            VmSubCmd::Shutdown(arg) => vmm::axvmm_shutdown_vm(arg).expect("Failed to shutdown VM"),
        },
    }
}
