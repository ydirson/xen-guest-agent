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

The agent might use `udev` to avoid creating a deamon.

### Features

The agent should gather some guest information, and writing them to the xenstore, so dom0 will be able to read it.

Current features (from `xe-guest-utilities`):

* Network metrics (vif ID, MAC, v4/v6 address)
* OS reporting
* Disk metrics
* Memory metrics (total, free)
* PV drivers version
* If ballooning is enabled

## What's next?

0. Present this repo to the community during next Xen community call
1. Build a PoC from scratch, for Linux, returning IP address
2. Discuss with the community about the PoC before going further (design, review etc.)
3. Merge in the main branch and start to advertise about it