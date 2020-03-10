Name: lustre_collector
Version: 0.2.12
# Release Start
Release: 1%{?dist}
# Release End
Summary: Scrapes Lustre stats and aggregates into JSON or YAML

License: MIT
Group: System Environment/Libraries
URL: https://github.com/whamcloud/%{name}
Source0: lustre_collector.tar.gz
ExclusiveArch: x86_64

%description 
%{summary}

%prep
%setup -c

%build

%install
mkdir -p %{buildroot}%{_bindir}
cp lustre_collector %{buildroot}%{_bindir}


%files
%attr(0755,root,root)%{_bindir}/lustre_collector

%changelog
* Fri Aug 16 2019 Joe Grund <jgrund@whamcloud.com> 0.1.0-1
- Initial build