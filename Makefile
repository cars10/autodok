lint:
	cargo fmt && cargo check && cargo clippy

run:
	cargo run

build:
	cargo build --release

docker_build:
	docker build . -t cars10/autodok:latest

docker_push:
	docker push cars10/autodok:latest

prod: docker_build docker_push
