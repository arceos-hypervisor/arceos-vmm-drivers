use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "axcli")]
#[command(about = "CommandLine Interface for ArceOS Hypervisor", long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
pub struct CLI {
    #[command(subcommand)]
    pub subcmd: CLISubCmd,
}

#[derive(Subcommand)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub enum CLISubCmd {
    /// Subcommands related to hypervisor itself.
    Hv {
        #[command(subcommand)]
        subcmd: HvSubCmd,
    },
    /// Subcommands related to the management of guest virtual machines.
    Vm {
        #[command(subcommand)]
        subcmd: VmSubCmd,
    },
}

#[derive(Subcommand)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub enum HvSubCmd {
    /// Enable arceos-hypervisor type1.5.
    Enable,
    /// Disable arceos-hypervisor type1.5.
    Disable,
}

#[derive(Subcommand)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub enum VmSubCmd {
    /// list the info of the vm
    List,
    /// Create guest VM according to config file.
    #[command(arg_required_else_help = true)]
    Create(VmCreateArgs),
    /// Boot guest VM according to VM id.
    #[command(arg_required_else_help = true)]
    Boot(VmBootShutdownArgs),
    /// Shutdown guest VM according to VM id.
    #[command(arg_required_else_help = true)]
    Shutdown(VmBootShutdownArgs),
}

#[derive(Debug, Args)]
pub struct VmCreateArgs {
    #[arg(value_name = "CONFIG_PATH")]
    pub config_path: String,
}

#[derive(Debug, Args)]
pub struct VmBootShutdownArgs {
    #[arg(value_name = "VMID")]
    pub vmid: u64,
}
