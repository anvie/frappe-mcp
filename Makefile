

all: target/debug/frappe_mcp

target/debug/frappe_mcp: src/**/*.rs
	cargo build

fmt:
	@@echo Formatting code...
	@@cargo fmt

test:
	cargo test

clean:
	@@echo Cleaning up...
	@@cargo clean

.PHONY: clean fmt

