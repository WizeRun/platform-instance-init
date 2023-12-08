ARG INSTANCE_INIT_VERSION

FROM rust:1.74 AS builder

ARG INSTANCE_INIT_VERSION

RUN mkdir -p /sandbox
WORKDIR /sandbox

COPY src /sandbox/src
COPY Cargo.toml /sandbox
COPY Cargo.lock /sandbox

RUN cargo build --release

FROM fedora:38 AS packager

RUN mkdir -p /sandbox
WORKDIR /sandbox

ARG INSTANCE_INIT_VERSION

RUN dnf install -y rpmdevtools

COPY --from=builder /sandbox/target/release/instance-init instance-init
COPY systemd/instance-init.service instance-init.service
COPY systemd/instance-init.target instance-init.target
COPY systemd/disable-sshd-keygen-if-instance-init-active.conf disable-sshd-keygen-if-instance-init-active.conf
COPY package.spec package.spec

RUN chown root:root instance-init && \
    chown 755 instance-init

RUN sed -i "s/INSTANCE_INIT_VERSION/${INSTANCE_INIT_VERSION}/g" package.spec && \
    rpmbuild --define "_topdir /sandbox" --build-in-place -bb package.spec

FROM scratch

ARG INSTANCE_INIT_VERSION
COPY --from=packager /sandbox/RPMS/x86_64/instance-init-${INSTANCE_INIT_VERSION}-1.fc38.x86_64.rpm instance-init-${INSTANCE_INIT_VERSION}-1.fc38.x86_64.rpm
