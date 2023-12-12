lint:
	cargo fmt && cargo check && cargo clippy

run:
	cargo run

build:
	cargo build --release

db_migrate:
	diesel migration run

db_reset:
	diesel database reset
	diesel migration run

db:
	docker compose up -d db
