# Project Guide

Marco Sparko is a Rust desktop application built with Dioxus. It lets users explore energy usage, costs, and bills from an Octopus Energy account. The application is designed around modules so other services can be added; Octopus Energy is the current module.

## Architecture

- `src/main.rs` parses command-line options and starts the desktop UI, or dispatches non-UI tasks such as fetching data, redacting profiles, and taking documentation screenshots.
- `src/lib.rs` contains shared application state, command-line argument handling, module registration, and the module interfaces.
- `src/components/` contains reusable Dioxus UI components. `src/views/` contains the app's pages and navigation.
- `src/octopus.rs` and `src/octopus/` implement the Octopus module: account and usage models, GraphQL access, data fetching, caching integration, and its pages and graphs.
- `src/data_set/` implements file-backed datasets and their refresh/caching strategies. `src/profile.rs`, `src/data_home.rs`, and `src/cache.rs` manage profiles and local data.
- Other top-level `src/` modules provide shared concerns such as help, logging, redaction, screenshots, date/time utilities, and window state.
- `graphql/octopus/` contains the Octopus schema and query documents. `build.rs` uses `sparko_graphql_builder` to generate typed GraphQL code during compilation. The builder and `sparko_graphql` crate are path dependencies in the sibling `../sparko_graphql` checkout.
- `assets/`, `icons/`, and `Dioxus.toml` hold UI and application bundle resources and configuration.
- `docs/` is the source for the mdBook user and developer documentation; `docs/SUMMARY.md` defines its navigation. `book/` contains the book's theme assets. See `docs/developers.md` for links to release, help-writing, screenshot, redaction, and API notes.
- `scripts/`, `gather_builds.sh`, `Cross.toml`, and release resources support release and cross-platform packaging.

## Development

Run these commands from the repository root:

```sh
cargo fmt --check
cargo check
cargo test
```

The desktop app can be run with the Dioxus CLI using `dx serve`. This project requires the sibling `sparko_graphql` checkout because its crates are referenced by local paths in `Cargo.toml`.

When changing GraphQL operations or the schema, update the source files in `graphql/octopus/`; generated code is produced by the build script. When changing app help, follow `docs/WritingHelp.md` so the in-app pages and mdBook stay in sync. For release procedures, see `docs/Release.md`.
