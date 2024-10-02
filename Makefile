IMG ?= anuragrajawat/httpfilter
TAG ?= test
CONTAINER_TOOL ?= docker

.PHONY: build
build:
	@cargo build --target wasm32-unknown-unknown --release

.PHONY: clean
clean:
	@cargo clean

.PHONY: imagex
imagex:
	$(CONTAINER_TOOL) build --platform=linux/arm64,linux/amd64 -t ${IMG}:${TAG} .

.PHONY: push
push:
	$(CONTAINER_TOOL) push ${IMG}:${TAG}
