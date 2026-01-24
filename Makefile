all: fmt build test


build:
	cargo build

fmt:
	cargo fmt

test:
	cargo test

DOCKER_COMPOSE=docker-compose -f docker/docker-compose.yaml --project-directory .
setup-e2e-env:
	$(DOCKER_COMPOSE) down
	$(DOCKER_COMPOSE) build
	$(DOCKER_COMPOSE) up --wait

e2e-tests:
	$(DOCKER_COMPOSE) down
	$(DOCKER_COMPOSE) build
	$(DOCKER_COMPOSE) up -d --wait
	cd tests/e2e_tests && npx playwright test -x
	$(DOCKER_COMPOSE) down

destroy-e2e-env:
	$(DOCKER_COMPOSE) down -v
