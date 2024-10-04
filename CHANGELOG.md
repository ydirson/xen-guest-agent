# Noteworthy changes for xen-guest-agent releases

The format is (loosely) based on [Keep a
Changelog](https://keepachangelog.com/en/1.0.0/)

## 0.5.0 - unreleased

### new features

* the RPM now enables and starts the service on first install
* the RPM now causes xe-guest-utilities to be uninstalled
  automatically

### bugfixes

* AlmaLinux and Rocky Linux guests are now reported as such, when
  build against os_info 3.8 or better
* the RPM now replaces xe-guest-utilities-latest too, not only
  xe-guest-utilities

### other noteworthy changes

* build now requires Rust 1.77
* build on FreeBSD does not require to set environment variables any
  more, now relies on pkg-config (requires "pkgconf" to build)

## 0.4.0 - 2024-01-29

### new features

* can be linked statically with libxenstore to distribute a more
  standalone binary (`-F static`).  Used for official Linux binary.

### bugfixes

* stale network information in xenstore is now removed on startup

### other noteworthy changes

* CI pipelines stopped producing binaries for EOL'd FreeBSD 12.4,
  switched to 13.2
* CI now produces an (unofficial) binary for FreeBSD with Netlink
  support

## 0.3.0 - 2023-12-15

### new features

* available and total guest memory are now collected in FreeBSD guests
* command-line flags `--stderr` and `--loglevel` were added to help
  troubleshooting

### behavior changes

* logs are now sent to syslog by default on Unix-like OS

### bugfixes

* the agent does not require the `libxenstore.so` symlink typically
  coming from Xen development package, only the runtime library
  package is now required
* VIF hot(un)plug is now properly handled

### other noteworthy changes

* executables and packages for supported guest platforms (currently
  Linux/Glibc and FreeBSD, both for x86_64 guests) are now available
  from Gitlab CI pipelines
* APT repositories (though not signed) are now available from Gitlab
  CI pipelines
* CI pipelines now testbuilds every commit in a merge request

## 0.2.0 - 2023-10-11

* initial public pre-release
