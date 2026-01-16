# Build the project in debug mode
build:
    cargo build

run *ARGS: build
    ./target/debug/vxectl {{ARGS}}


# Build the project in release mode
build-release:
    cargo build --release

# Clean the project
clean:
    cargo clean

# Run linting and formatting
lint:
    cargo fmt
    cargo clippy --all -- -D warnings