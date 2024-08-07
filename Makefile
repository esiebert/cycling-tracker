run:
	cargo run

ui:
	grpcui -insecure localhost:10000

fmt:
	cargo fmt
	cargo clippy

db-migrate:
	sqlx migrate run

db-populate:
	sqlite3 ct.db < ./data/sql/populate_db.sql

build:
	cargo build

build-image:
	docker build . -t cts

run-container:
	docker run -p 10000:10000 --env-file .env --name cts esiebert/ct

test:
	# Redis has to be started manually because setting up tests with
	# testcontainers didn't work
	docker compose up -d cts-redis
	-cargo test
	docker compose down cts-redis
