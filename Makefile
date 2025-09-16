

VERSION=$$(grep "^version = " Cargo.toml | sed -E "s/version = \"(.+)\"/\1/")

all: target/debug/frappe_mcp

target/debug/frappe_mcp: src/**/*.rs
	cargo build

fmt:
	@@echo Formatting code...
	@@cargo fmt

test:
	cargo test

version: ## Bump patch version in Cargo.toml and commit changes
	@bash -ec ' \
		old=$$(grep "^version = " Cargo.toml | sed -E "s/version = \"(.+)\"/\1/"); \
		IFS=. read -r a b c <<< "$$old"; \
		new=$$(printf "%s.%s.%d" $$a $$b $$((c+1))); \
		sed -i.bak -E "s/^version = \".*\"/version = \"$$new\"/" Cargo.toml; \
		git add Cargo.toml; \
		echo "Version bumped from $$old to $$new"; \
	'

commit-version:
	@git commit -m "Bump version v$(VERSION)" Cargo.toml

clean:
	@@echo Cleaning up...
	@@cargo clean

build-linux-amd64:
	@@echo Building for Linux AMD64...
	@@cargo zigbuild --target x86_64-unknown-linux-gnu --release

build-linux-arm64:
	@@echo Building for Linux ARM64...
	@@cargo zigbuild --target aarch64-unknown-linux-gnu --release


.PHONY: clean fmt build-linux-amd64 build-linux-arm64 version commit-version test

