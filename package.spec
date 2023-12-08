Name:           instance-init
Version:        INSTANCE_INIT_VERSION
Release:        1%{?dist}
Summary:        A very minimalist cloud-init-replacement

License:        Apache-2.0

%description

A very minimalist cloud-init replacement

%install

mkdir -p %{buildroot}/usr/bin
cp -p instance-init %{buildroot}/usr/bin

mkdir -p %{buildroot}/lib/systemd/system
cp -p instance-init.service %{buildroot}/lib/systemd/system/instance-init.service
cp -p instance-init.target %{buildroot}/lib/systemd/system/instance-init.target

mkdir -p %{buildroot}/etc/systemd/system/sshd-keygen@.service.d
cp disable-sshd-keygen-if-instance-init-active.conf %{buildroot}/etc/systemd/system/sshd-keygen@.service.d/disable-sshd-keygen-if-instance-init-active.conf

%post

/bin/systemctl enable mini-cloud-config.service

%preun

/bin/systemctl --no-reload disable instance-init.service   >/dev/null 2>&1 || :

%postun

/bin/systemctl daemon-reload >/dev/null 2>&1 || :

%files
/usr/bin/instance-init
/etc/systemd/system/sshd-keygen@.service.d/disable-sshd-keygen-if-instance-init-active.conf
/lib/systemd/system/instance-init.service
/lib/systemd/system/instance-init.target
