# Snow scanner

This project name is inspired by the Netflix series "The Snowpiercer"

## Run it

```sh
SERVER_ADDRESS="127.0.0.1:8777" \
DB_URL="mysql://db-user:db-pass@db-server/db-snow-scanner" \
STATIC_DATA_DIR="$PWD/../data" \
cargo run --release
```

## Run in production

The env file located at `/etc/snow-scanner/.env`:

```env
# Your public IP
SERVER_ADDRESS="[2a10:ffff:ff:ff:fff::1]:80"
DB_URL="mysql://db-user:db-pass@db-server/db-snow-scanner"
STATIC_DATA_DIR="/usr/share/snow-scanner/data"
# Adjust this
ROCKET_LOG_LEVEL="debug"
ROCKET_PROFILE="debug"
# Setup TLS
ROCKET_TLS='{certs="/etc/ssl/certs/cert.pem",key="/etc/ssl/private/key.pem", mutual={ca_certs="/etc/ssl/certs/cloudflare.crt",mandatory=true}}'
```

## Make a new release

Pre-requirements: `cargo install cargo-deb`

- Bump `Cargo.toml`
- Commit it
- Run `cd snow-scanner`
- Run `cargo update`
- Commit the changes to the lockfile
- Run tests `cargo test`
- Run `cargo deb`
- Run `./debian/upload.sh target/debian/snow-scanner_1.1.0-1_amd64.deb`
- Run `git tag -a -s -m "snow-scanner/1.1.0" snow-scanner/1.1.0`
- Check `debdiff ../../snow-scanner_1.0.0-1_amd64.deb target/debian/snow-scanner_1.1.0-1_amd64.deb`
