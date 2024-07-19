run:
	cargo run

ui:
	grpcui -insecure localhost:10000

fmt:
	cargo fmt
	cargo clippy
