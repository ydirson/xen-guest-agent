# Draft/proposed interface for guest tools data

This is a proposal about sets of data to be collected for publication
outside of the guest for administration needs, general principles on
how we want to organize/structure those data, and proposed options for
layouts.

This only covers the data collection aspect of guest tools, and makes
no hypothesis on whether their scope will be extended in the future.


## Questions to be decided first

### Scope of data collection

Existing guest tools collect and expose several categories of
informations:

* static information for guest classification to help the use managing
  through the toolstack
  * OS kind, version, etc
  * IP addresses for VIF and SR-IOV virtual-function NICs
  * mount points, partition sizes, filesystem types, etc
* information to help the toolstack manage the guest
  * whether PV drivers/tools are installed
  * whether the guest supports memory balloonning, and ballooning
    driver has reached `~/memory/target` (which it does not really
    know)
* monitoring data
  * amount of available memory
  * block-device usage (available fs space)
  * timestamp of last xenstore update

Those "monitoring data", are typically collected in an enterprise
setting by dedicated monitoring tools.  Those tools however typically
assume network reachability of the monitored resources, and we can do
better with a virtualized environement.  But better rarely ryhmes with
"completely different and from scratch".  Providing a transport for
existing monitoring systems could be a solution (e.g. Nagios/whatever
over vchan?).

Criteria to decide what data to expose ourselves needs to include
whether there is any practical advantage over existing monitoring
agents.

### Communication channel

Current implementations use Xenstore to expose the collected data.
This seems adequate for rarely-changing data (e.g. network config),
but alternatives can be considered for other kinds of data that highly
depend on the usage made of them (e.g. free/used memory).


## (Tentative) basic principles

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
  allowed as long as they live under a path that makes it clear it is
  not generic


## Data to be exposed

### Protocol identification

* explicit identification that an agent is making use of this protocol
* version of the protocol used for data layout

Both can be exposed by a single key/value.

### Guest identification

* OS name, version
* Kernel version (for UNIX-like OS at least; other cases?)

### Infrastructure under guest control

* (V)NIC network config

### Monitoring

* (V)NIC link status


## Structure proposals

Note the proposed structures here all live under `data/`, but a
separate dedicated hierarchy could make sense.

### Static info

#### Proposal 1

```
data/
  schema = "0.1.0"
  os/
    name = "Debian GNU/Linux"
    version = "11.6"
    unix/
      kernel-version = "5.10.0-21-amd64"
```

### Network information

We need IPv4/IPv6 configuration from at least some network devices in
the VM.  The devices of interest will be devices connecting the guest
to the outside world.  This includes at least be VIF's, SRIOV VF's,
network devices exposed by PCI/USB passed-through devices, and bridges
including one of those as a member.

For unique device identification, we have several options:
- rely on MAC address (though some hardware allow changing it, and
  there seems to be use-cases for several MACs on a given NIC, for
  which we should provide some details if using it to definitely rule
  out this option)
- rely on guest naming (most guests have mechanisms for interface
  name stability nowadays, though it is still possible to make them
  change)
- rely on a hardware topology (this is what is done, to some extent,
  by `xe-guest-utilities` for VIF's by using their VIF index)
- rely on an identifier the guest guaranties to be unique during the
  guest's lifetime (e.g. POSIX interface ID, but there is likely no
  general guarantee that an arbitrary OS will have something similar?)

Hardware topology seems a promising option but requires to put more
thought in it first.  The following proposals use guest naming, but
MAC address can be use nearly as easily, by mangling them to avoid the
use of ":".

Each device may have several IPv4 and IPv6 addresses.  If we want each
of those addresses so they have a predictable path (as opposed to
proposal 1 below).

#### Proposal 1

Number IP addresses to overcome the lack of mutable list values in
Xenstore.  This is what similar to [the `~/attr/vif/`
namespace](https://xenbits.xen.org/docs/unstable/misc/xenstore-paths.html#attrvifdevidname-string-w)
today applied by e.g. `xe-guest-utilities` (here moved to `~/data/` to
comply with the "single hierarchy" principle), but violates the
"predictable path" basic principle exposed earlier.

```
data = ""
  net = ""
    eth0 = "11:22:33:44:55:66"
      ipv4 = ""
        0 = "10.0.0.4"
        1 = "172.16.0.5"
      ipv6 = ""
        0 = "1234::4578:abcd"
```

Problems with such a layout include:

* removing an address (e.g. 10.0.0.4 here) either produces a
  numeration hole (in which case we may not want to reuse previous
  indices, and will face arbitrarily-growing address indices), or
  requires rewriting all addresses with higher indices, (which seems
  worse, causing the Xenstore watch operation to be much more
  complicated to use properly)

* the paths not being predictable, several Xenstore lookups will be
  necessary to determine which entry pertains to a given address.  The
  agent will then need to maintain a cache of the mapping to work
  efficiently (which then complicates restarting the agent process).

#### Proposal 2

Use IP addresses as keys.  Sort of, because "." and ":" are not valid
in a xenstore path, so we have to substitute a valid character that's
not otherwise valid inside an IP address, for which the best
candidates are "-_".

This proposal also shows a possible solution to address multiple MAC
addresses per interface, similar to the current `addr/vif/` layout.

```
data = ""
  net = ""
    eth0 = ""
      mac = ""
        11_22_33_44_55_66 = ""
      ipv4 = ""
        10_0_0_4 = ""
        172_16_0_5 = ""
      ipv6 = ""
        1234__4578_abcd = ""
```

Possible alternatives, depending on how much we want the literal values
to be usable by humans and simpler scripts:

- add human-readable version of IP address as value for the mangled key
- use hexadecimal form as key to avoid the need for mangling; since it
  makes the key less useful for humans, the human-readable version as
  value could be useful too (but this makes the data redundant, which
  can pose its own issues)

```
data = ""
  net = ""
    eth0 = ""
      mac = ""
        112233445566 = "11:22:33:44:55:66"
      ipv4 = ""
        0A000004 = "10.0.0.4"
        AC100005 = "172.16.0.5"
      ipv6 = ""
        1234000000000000000000004578ABCD = "1234::4578:abcd"
```

#### Proposal 3

Interface names are not necessarily stable, as they can be renamed,
and the OS usually has a non-recycled unique identifier for interfaces
(an index for POSIX systems, [a GUID for Windows
systems](https://learn.microsoft.com/en-us/windows/win32/network-interfaces)).
We could thus use them in the "Proposal 2" schema, which will require
less bookkeeping in the guest agent, and avoid unnecessary xenstore
churn:

```
data = ""
  net = ""
    42 = "eth0"
      mac = ""
...
```
