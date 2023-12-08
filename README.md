# Instance Init

instance-init is a simple tool that run on cloud instance startup.
It's used to initialize specific settings of the instance (e.g. hostname, ssh keys, etc.).

Configuration is pulled from the cloud provider metadata service.

## Usage

instance-init starts by reading the configuration from the cloud provider metadata service.
Then it sets up the instance hostname (transiently) and tries to connect to the API of the cloud provider 
to grab additional information (e.g. is the instance part of a pool?, is the pool fully provisioned ? etc.).

At this point, instance-init will install SSH keys on the instance and will start the SSH daemon.

Currently, only the [Exoscale public cloud](https://www.exoscale.com) is supported.

### Why not cloud-init?

Cloud-init is a great tool but is not suitable for instance templates with read-only root filesystems.
It also doesn't help with keeping minimal images since it requires a python runtime and a bunch of dependencies.

We try to solve this problem by providing a very minimalist tool that is both minimal and self sufficient.


## Future plans

Future plan is to add support for more complex initialization scenarios, e.g.:
- TLS certificates provisioning (e.g. from Hashicorp Vault or lightstep)
- cluster initialization (e.g. etcd)
- vpn mesh setup (e.g. wireguard)
- integrated API

Future plan is also to add support for other cloud providers.