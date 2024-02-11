lint:
	cargo fmt && cargo check && cargo clippy

run:
	cargo run

build:
	cargo build --release

docker:
	docker build . -t autodok
