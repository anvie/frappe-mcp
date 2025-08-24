

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

build-linux-amd64:
	@@echo Building for Linux AMD64...
	@@cargo build --release --target x86_64-unknown-linux-gnu

build-linux-arm64:
	@@echo Building for Linux ARM64...
	@@cargo build --release --target aarch64-unknown-linux-gnu

.PHONY: clean fmt build-linux-amd64 build-linux-arm64

