install:
	cargo install cargo-edit
	cargo add actix-web
	cargo add actix-cors
	cargo add serde_json
	cargo add serde --features derive
	cargo add chrono --features serde
	cargo add env_logger
	cargo add dotenv
	cargo add uuid --features "serde v4"
	cargo add sea-orm --features "sqlx-mysql runtime-async-std-native-tls macros"
	cargo add argonautica
	cargo add jwt
	cargo add actix-web-httpauth
	cargo add hmac
	cargo add sha2
	cargo add reqwest --features "json"
	cargo add tokio --features "full"
	cargo add anyhow
	cargo add thiserror
	cargo install cargo-watch

build: 
	cargo build

run:
	cargo run

dev:
	cargo watch -x run

# Need to have "sea-orm-cli" installed prior with "cargo install sea-orm-cli"
migrate_init:
	sea-orm-cli migrate init

migrate_run:
	sea-orm-cli migrate up

migrate_rollback:
	sea-orm-cli migrate down

