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
