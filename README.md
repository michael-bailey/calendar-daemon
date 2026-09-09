# caldav-server

A CalDAV server targeting Apple Calendar (macOS / iOS), built on Tokio + axum.

## Structiure
This project utilises cargo workspaces, to build multiple application binaries.

```
src/
├── ctl         # System control, configuration, and helper cli tool 
├── lib         # Shared models and code for each
├── main        # Calendard server implemention
└── test        # Integration tests
```

## Setup

### 1. Configure environment

```bash
export BASE_URL="http://127.0.0.1:3000"
export DATABASE_URL="./database.sqlite"
export CALDAV_USERNAME="alice"
export CALDAV_DISPLAY_NAME="Alice"
# Generate with: cargo run -- passwd <your password>
export CALDAV_PASSWORD_HASH='$2b$12$...'
export BIND_ADDR="127.0.0.1:3000"
```

The app accepts either `./database.sqlite` or a full SQLx URL such as
`sqlite:./data/calendar.db?mode=rwc`. It creates the SQLite parent directory,
runs migrations, and creates a default `Personal` calendar on first startup.

### 2. Run

```bash
cargo run
```

To generate a password hash:

```bash
cargo run -- passwd yourpassword
```

## Testing

```bash
# Unit + integration tests (uses in-memory store, no setup needed)
cargo test

# With tracing output
RUST_LOG=caldav_server=debug cargo test -- --nocapture
```

## Apple Calendar setup

1. macOS: **System Settings → Internet Accounts → Add Account → Other → CalDAV**
2. Set server address to your `BASE_URL` host, e.g. `127.0.0.1:3000`
3. Enter username and password

iOS: **Settings → Calendar → Accounts → Add Account → Other → Add CalDAV Account**

## What's implemented

### Mandatory
- [x] Apple discovery chain (`.well-known` → principal → home-set)
- [x] `/.well-known/caldav` and `/.well-known/calendar` redirects
- [ ] `PROPFIND` on root, principal, home-set, calendar, object
- [ ] `OPTIONS` with WebDAV/CalDAV capability headers
- [ ] `GET` / `PUT` / `DELETE` for `.ics` objects
- [ ] ETags + conditional requests (`If-Match`, `If-None-Match`)
- [ ] `REPORT` stub (returns all objects — full calendar-query parsing TODO)
- [ ] `MKCALENDAR` for creating calendars from clients
- [ ] Basic auth middleware with bcrypt password verification
- [ ] SQLite store with migrations

### Future ideas 
- [ ] In-memory store for tests
- [ ] Integration test suite
- [ ] Full `REPORT` XML parsing (calendar-query date filters, calendar-multiget)
- [ ] WebDAV sync-collection (RFC 6578) — efficient delta sync
- [ ] TLS via `axum-server` + rustls
- [ ] Kubernetes deployment manifests

## Disclaimers
This project heavily utilised code generative AI to create a 'functional' version
I'm in the process of refactoring it, as it's implementation was barely 'passable'

This includes:
- Restructuring data to be database first rather than bulk CalDAV file formats.
- Implementing, compliant CalDAV protocols for accounts and authentication.
- Redesiging domain models, to be correct and eliminate as many invalid states as possible