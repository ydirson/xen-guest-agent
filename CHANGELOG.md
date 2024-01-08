# Noteworthy changes for xen-guest-agent releases

The format is (loosely) based on [Keep a
Changelog](https://keepachangelog.com/en/1.0.0/)

## 0.4.0 - unreleased

### other noteworthy changes

* CI pipelines stopped producing binaries for EOL'd FreeBSD 12.4,
  switched to 13.2

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
