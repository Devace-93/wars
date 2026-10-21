.PHONY: up down logs db check server desktop tui android ios-ipa
COMPOSE=docker compose -f deploy/docker-compose.yml --project-name wars

up:            ## build + start Postgres and the server on :8086
	$(COMPOSE) up -d --build
down:
	$(COMPOSE) down
logs:
	$(COMPOSE) logs -f --tail=100
db:            ## start only Postgres (for `make server`)
	$(COMPOSE) up -d postgres
check:         ## fmt + clippy + tests
	./scripts/check.sh
server: db     ## run the server from source against the dev Postgres
	DATABASE_URL=postgres://wars:wars@127.0.0.1:5486/wars cargo run -p wars-server
desktop:
	cargo run --release -p wars-desktop
tui:
	cargo run --release -p wars-tui
android:
	cd apps/android && ./gradlew assembleDebug
