all: fmt lint build test

fmt:
	cargo fmt

lint:
	cargo clippy

build:
	cargo build

test:
	cargo test

DOCKER_COMPOSE=docker-compose -f docker/docker-compose.yaml --project-directory .
setup-e2e-env: build
	$(DOCKER_COMPOSE) down
	$(DOCKER_COMPOSE) build
	$(DOCKER_COMPOSE) up --wait

e2e-tests: build
	$(DOCKER_COMPOSE) down
	$(DOCKER_COMPOSE) build
	$(DOCKER_COMPOSE) up -d --wait
	cd tests/e2e_tests && npx playwright test -x
	$(DOCKER_COMPOSE) down

destroy-e2e-env:
	$(DOCKER_COMPOSE) down -v
