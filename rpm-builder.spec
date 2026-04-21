%bcond check 0

Name:           rpm-builder
Version:        0.4.0
Release:        %{?autorelease}%{!?autorelease:1%{?dist}}
Summary:        Simple Rust CLI utility for creating simple RPMs

License:        Apache-2.0
URL:            https://github.com/rpm-rs/rpm-builder
Source0:        https://crates.io/api/v1/crates/%{name}/%{version}/download#/%{name}-%{version}.crate
Source1:        %{name}-%{version}-vendor.tar.gz

BuildRequires:  rust >= 1.85
BuildRequires:  cargo

%if 0%{?rhel} == 9
%global _debugsource_template %{nil}
%endif

%description
A simple Rust CLI utility for creating RPM packages without spec files.

%prep
%autosetup -n %{name}-%{version}
tar -xf %{SOURCE1}
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
CARGO_PROFILE_RELEASE_STRIP=none cargo build --release --frozen

%install
install -Dpm 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%if %{with check}
%check
cargo test --release --frozen
%endif

%files
%license LICENSE
%doc README.md
%{_bindir}/rpm-builder

%changelog
* Mon Apr 20 2026 Daniel Alley <dalley@redhat.com> - 0.4.0-1
- Update to 0.4.0
