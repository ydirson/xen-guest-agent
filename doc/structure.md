# Draft interface from guest tools data

This is a proposal about sets of data to be collected for publication
outside of the guest for administration needs, general principles on
how we want to organize/structure those data, and proposed options for
layouts.

This only covers the data collection aspect of guest tools, and makes
no hypothesis on whether their scope will be extended in the future.


## Questions to be decided first

### Scope of data collection

A number of information made available by existing guest tools are
typical monitoring data, typically collected in an enterprise setting
by dedicated monitoring tools.  This include info about block-device
use (fs, mountpoint, available fs space), and "free memory as seen by
OS kernel", as well as other prospective features such as availability
of software updates.

Those tools however typically assume network reachability of the
monitored resources, and we can do better with a virtualized
environement.  But better rarely ryhmes with "completely different and
from scratch".  Providing a transport for existing monitoring
systems could be a solution (e.g. Nagios over vchan?).

Criteria to decide what data to expose ourselves needs to include
whether there is any practical advantage over, say, a Nagios probe.


## Basic principles

* data layout should be semver-versionned, so clients can make a single
  check to determine if they know how to read it and where to locate
  the info they need

* at least data changing infrequently is exposed using Xenstore at the
  initiative of the guest tools; data changing more more often could
  use a different communication mechanism (eg. based on Matias' work
  for performance data), and this will be explored in a later phase

* all guest-tools data is exported under a unique Xenstore hierarchy
  for each guest, refered to as "data tree" in this document

* data should be stored in a predictable place ("data path") in the
  data tree (e.g. use of arbitrary identifiers like numeric indices
  should be avoided)

* data path should be as much as possible chosen using generic
  wording, avoiding OS-specific idioms that would make it awkward to
  apply to another OS

* OS-specific data, like data specific to a given hardware, are
  allowed as long as they leave under a path that makes it clear it is
  not generic


## Data to be exposed

### Protocol identification

* explicit identification of this protocol
* data layout version

### Guest identification

* OS name, version
* Kernel version (for UNIX-like OS at least; other cases?)

### Infrastructure under guest control

* (V)NIC network config

### Monitoring

* (V)NIC link status


## Structure proposals

