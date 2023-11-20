check:
	cargo fmt && cargo check && cargo clippy

run:
	cargo run

build:
	cargo build --release

db:
	docker compose up -d db
