# AxDaemon

Daemon process of arceos-hypervisor for VMM support.

It has to be built on rust-nightly due to the dependency of [axerrno](https://crates.io/crates/axerrno).

It has to be run under **sudo** privilege for mmap related operations.

## Howto

* Build
```bash
cargo build --release
```

* Use

```bash
ubuntu@ubuntu:~/axdaemon$ sudo ./target/release/axdaemon
AxDaemon: Daemon process of arceos-hypervisor for VMM support

Usage: axdaemon <COMMAND>

Commands:
  init  Start daemon
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```