Name:           xen-guest-agent
Version:        0.2.0
%define upstreamversion %{version}
Release:        0%{dist}
Summary:        Agent for Xen virtual machine

License:        AGPL-3.0-only
URL:            https://gitlab.com/xen-project/xen-guest-agent/

# main "source" is binary built with Rustup
Source0:        xen-guest-agent
Source1:        xen-guest-agent.service

BuildRequires:  systemd-devel

Requires: xen-libs
Conflicts: xe-guest-utilities

%global _description %{expand:
%{summary}.}

%define _debugsource_template %{nil}
%debug_package

%description %{_description}

%install
%{__install} -m755 -d %{buildroot}%{_sbindir}
%{__install} -m755 %{SOURCE0} %{buildroot}/%{_sbindir}/xen-guest-agent
%{__install} -m 755 -d %{buildroot}%{_unitdir}
%{__install} -m 644 %{SOURCE1}  %{buildroot}%{_unitdir}

%files
%{_sbindir}/xen-guest-agent
%{_unitdir}/xen-guest-agent.service

%post
%systemd_post xen-guest-agent.service

%postun
%systemd_postun xen-guest-agent.service

%changelog
* Mon Nov 06 2023 Yann Dirson <yann.dirson@vates.tech> - 0.2.0-0
- Upstream package from release binaries
