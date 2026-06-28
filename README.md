# caldav-server

A CalDAV server targeting Apple Calendar (macOS / iOS), built on Tokio + axum.

## Architecture

```
src/
├── domain/          # Pure data types — Calendar, CalendarObject, Principal
├── store/
│   ├── calendar_store.rs   # CalendarStore trait (the DI contract)
│   ├── sqlite.rs           # SQLite backend (production)
│   └── memory.rs           # In-memory backend (tests)
├── caldav/
│   ├── discovery.rs        # .well-known, principal, home-set PROPFIND
│   ├── calendar.rs         # Calendar collection handlers
│   ├── object.rs           # .ics GET / PUT / DELETE / REPORT
│   ├── propfind.rs         # General PROPFIND dispatcher
│   └── xml.rs              # WebDAV XML response builders
├── auth.rs          # Basic auth middleware
├── router.rs        # Route table + method dispatch
├── app_state.rs     # AppState — DI root
├── config.rs        # Environment-based config
└── error.rs         # StoreError + AppError
```

### Dependency injection

The `CalendarStore` trait is the seam between handlers and persistence.
`AppState` holds an `Arc<dyn CalendarStore>` so any implementation can
be swapped in — SQLite for production, `InMemoryStore` for tests.

```rust
// Production
let store = Arc::new(SqliteStore::new(pool).await?);

// Tests
let store = Arc::new(InMemoryStore::default());
```

## Setup

### 1. Configure environment

```bash
export BASE_URL="http://127.0.0.1:3000"
export DATABASE_URL="./database.sqlite"
export CALDAV_USERNAME="alice"
export CALDAV_DISPLAY_NAME="Alice"
# Generate with: cargo run -- passwd yourpassword
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

- [x] Apple discovery chain (`.well-known` → principal → home-set)
- [x] `/.well-known/caldav` and `/.well-known/calendar` redirects
- [x] `PROPFIND` on root, principal, home-set, calendar, object
- [x] `OPTIONS` with WebDAV/CalDAV capability headers
- [x] `GET` / `PUT` / `DELETE` for `.ics` objects
- [x] ETags + conditional requests (`If-Match`, `If-None-Match`)
- [x] `REPORT` stub (returns all objects — full calendar-query parsing TODO)
- [x] `MKCALENDAR` for creating calendars from clients
- [x] Basic auth middleware with bcrypt password verification
- [x] SQLite store with migrations
- [x] In-memory store for tests
- [x] Integration test suite

## What's next

- [ ] Full `REPORT` XML parsing (calendar-query date filters, calendar-multiget)
- [ ] WebDAV sync-collection (RFC 6578) — efficient delta sync
- [ ] TLS via `axum-server` + rustls
- [ ] Kubernetes deployment manifests
