set dotenv-load
set positional-arguments

run *ARGS:
    cargo run -- "$@"

test *ARGS:
    cargo test -- --nocapture "$@"

build:
    cargo build

install:
    cargo install --path . --locked

check:
    cargo check
