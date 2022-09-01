.PHONY: build
build:
	cargo build --target-dir ./target --target wasm32-unknown-unknown --release

.PHONY: stream
stream:
	substreams run -p -e localhost:18015 substreams.yaml block_output_to_token --start-block 122 --stop-block +1

.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"