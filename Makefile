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
	cargo test

setup-env-linux:
	sudo apt install -y protobuf-compiler libssl-dev pkg-config
	cargo install sqlx-cli --version=0.8.0 sqlx-cli --no-default-features --features sqlite

setup-env-macos:
	brew install protobuf-compiler
	cargo install sqlx-cli --version=0.8.0 sqlx-cli --no-default-features --features sqlite
