lint:
	cargo fmt && cargo check && cargo clippy

run:
	cargo run

build:
	cargo build --release

docker_build:
	docker build . -t autodok

docker_push:
	docker push autodok:latest ghcr.io/cars10/autodok:latest

prod: docker_build docker_push
