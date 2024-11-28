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


### Tests

build_test:
	docker compose -f compose.test.yml build test

test: _start_test_docker_server
	docker compose -f compose.test.yml run --rm test cargo test -- --nocapture

test_bash: _start_test_docker_server
	docker compose -f compose.test.yml run --rm -it test bash

_start_test_docker_server:
	docker compose -f compose.test.yml up -d --remove-orphans server
