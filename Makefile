.PHONY: build
build:
	@cargo build --target wasm32-unknown-unknown --release

.PHONY: clean
clean:
	@cargo clean
