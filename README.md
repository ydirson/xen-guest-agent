# xen-guest-agent

The `xen-guest-agent` repository will be the new upstream for the Xen agent running in Linux/BSDs (POSIX) VMs.

Goals are:
* providing a unified upstream source repo for distro maintainers
* writing it from scratch to make it simple as possible
* building it with an incremental approach (Linux support first)
* being community driven
* splitting clearly the agent from libs (xenstore-read/write…)
* reducing the burden to package it for distro's

## General design

### Features

The agent gathers some guest information, and writes them to xenstore
so tooling in dom0 can read it.  The default behavior is to be
compatible with the XAPI toolstack as currently used in XCP-ng and
Citrix Hypervisor / Xenserver, and thus roughly follow what
`xe-guest-utilities` is doing.

We want it to become more largely useful; for this the collection
scope and publication structure still need to stabilize, and proposals
are discussed in [a separate document](doc/structure.md).

Current features:

* Network metrics (vif ID, MAC, v4/v6 address)
* OS reporting
* Memory metrics (total, free)
* Support for squeezed ballooning controller on toolstack size

Some features to consider (from `xe-guest-utilities`):
* Disk metrics
* "PV drivers version"

## Rust prototype

### How to build

Make sure you have a Rust toolchain available.  The base command is simply:

```
cargo build
```

By default it will (attempt to) build a `netlink` network collector
(which is only known to work on Linux, and should work on FreeBSD 13.2
and later), and a `xenstore` data publisher.  Today the only
alternative to the `netlink` network collector is a "no-op" one, and
similarly the only alternative to the `xenstore` data publisher is a
"mostly-no-op" one publishing to stdout.

Building with the `--no-default-features` flag with select those
"no-op" implementations instead.  Selecting only one "no-op"
implementation requires starting similarly with no feature, and adding
those you want with `-F`.

For example to test the xenstore publisher without netlink support,
you can use:

```
cargo build --no-default-features -F xenstore
```

If you have `libxenstore` installed in a non-standard place (this
includes `/usr/local` on FreeBSD), set the following environment
variables when running `cargo`:

```
BINDGEN_EXTRA_CLANG_ARGS=-I/usr/local/include
RUSTFLAGS=-L/usr/local/lib
```

### How to run

The only way to tune the behavior currently is by using environment
variables:

* `XENSTORE_SCHEMA`: select the schema to use for publishing of data in Xenstore.
  Possible values:
  * `std`: (default value) network info according to [the xenstore-path
    doc](https://xenbits.xen.org/docs/unstable/misc/xenstore-paths.html#domain-controlled-paths),
    the rest compatible with what XAPI currently expects
  * `rfc`: alternate layout as proposed in [a separate document](doc/structure.md)

### Current state, limitations

* we decided to play with Rust [for various
  reasons](https://xcp-ng.org/blog/2023/03/17/bringing-rust-to-the-xen-project/)
* the prototype was written with Rust async programming features, both
  because it is the natural way to use the netlink crates, and for
  experimentation purposes to get a grasp on Rust's take on the
  subject
* it is written around the idea of various information collectors
  (today: OS, kernel, network, memory) and publishers (today:
  XenStore, with compatibility for today's Xenserver tool or with an
  alternative "rfc" structure), with additional helpers
  (identification of whether a NIC is a VIF, with a rough /sys-based
  implementation for Linux, and a rough untested implementation for
  FreeBSD based on interface name)
* access to Xenstore is done using Mathieu Tarral's early-stage work
  on [Rust Xenstore bindings](https://lib.rs/crates/xenstore-rs),
  which we [enhanced with write
  access](https://github.com/Wenzel/xenstore/pull/10).  An official
  Rust Xenstore API will be required at some point; another candidate
  would be Starlab's [pure-Rust libxenstore
  implementation](https://github.com/starlab-io/xenstore-rs), which is
  also in a prototype state
* the Xenstore publisher's "std" schema exposes only information
  currently identified as used by the XAPI/XenOrchestra stack (notably
  MAC addresses for VIFs are not exposed yet, even though they appear
  in [the xenstore-path
  doc](https://xenbits.xen.org/docs/unstable/misc/xenstore-paths.html#domain-controlled-paths))
* the Linux VIF-identification implementation is simplistic, skipped
  the SR-IOV case
* the fallback VIF-identification implementation (expectedly) causes
  no NIC to be reported at all
* the VIF-identification mechanism is currently disabled by editing
  the code (`ONLY_VIF` flag), and this interferes with the
  `XENSTORE_SCHEMA=rfc` mode, which would otherwise be able to publish
  information about non-VIF network interfaces
* for ease of experimentation, alternative implementations of
  collectors and publishers are selectable at compile-time, though a
  number of those choices will make sense as runtime options
* similarly, some behaviours are tunable by modifying flags in the code
* a single implementation for network-configuration info listening to
  netlink events is provided, with as alternative a no-op collector
  for un-/not-yet- supported OS
* error handling is typical of a proto (but Rust will make fixing this
  unexpectedly easy, having forced the use of well-identified
  constructs like `.unwrap()` and `?` right from this early stage)


## What's next?

0. Discuss with the community about the PoC before going further (design, review etc.)
1. Converge with an implementation in the main branch and start to advertise about it
