# build and run server on save
dev:
    cargo watch --poll -q -c -w src/ -x "run --bin youtube-summarizer-server --features dotenv"

# build and run example query on save
example:
    cargo watch --poll -q -c -w examples/ -x "run --example quick_dev -- --nocapture"

# format rust, justfile, and markdown
format:
    cargo fmt --all
    just --fmt --unstable
    npx -y prettier './**/*.{md,yaml}' --write

# check formatting for rust, justfile, and markdown
format-check:
    cargo fmt --all -- --check
    just --fmt --unstable --check
    npx -y prettier './**/*.{md,yaml}' --check

# lint rust
lint:
    RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features 

sqlx:
    cargo sqlx prepare --workspace -- --all-targets --all-features

test-cov:
    cargo llvm-cov
