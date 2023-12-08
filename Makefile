KANIKO_CMD = docker run -t --rm -v $(PWD):/workspace -w /workspace gcr.io/kaniko-project/executor:v1.17.0

VERSION := $(shell git describe --tags --always --dirty)
SHELL := /bin/bash
.SHELLFLAGS := -e -c

.PHONY: all
all: rpms

.PHONY: clean
clean:
	@rm -rf *.rpm
	@rm -rf *.tar

.PHONY: package
package:
	$(KANIKO_CMD) --dockerfile=Dockerfile --no-push --tar-path instance-init-image.tar --build-arg INSTANCE_INIT_VERSION=$(VERSION)
	tar xf instance-init-image.tar --wildcards '*.tar.gz' --to-stdout | tar xzf - --wildcards '*.rpm'
	sha256sum -b *.rpm > SHA256SUMS
