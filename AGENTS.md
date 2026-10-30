# Notes for Coding Agents

Marco Sparko is a Rust desktop app, built with [Dioxus](https://dioxuslabs.com), for seeing a household's energy: its
use, costs and bills from Octopus Energy, the one module so far. People's guides are in
[docs/developers.md](docs/developers.md), which links to the notes on releases, help, redaction and screenshots.

## Layout

- `src/main.rs` and `src/lib.rs`: the command line, the shared context, and the `Module` trait which each source of
  data implements, registered in `ModuleRegistrations`.
- The Octopus module: `src/octopus.rs` and `src/octopus/` (GraphQL, pages and graphs).
- `src/data_set/`: the file-backed caches the modules' data is kept in, each with its own rules for when to fetch again.
- `src/components/` and `src/views/`: shared Dioxus components, and the app's own pages.
- `src/profile.rs`, `src/cache.rs`, `src/data_home.rs`: profiles, and where their settings and caches are kept.
- `src/redact.rs`: redacted, offline copies of a profile, for the docs and for testing without an account.
- `graphql/octopus/`: the Octopus schema and the queries, from which `build.rs` generates the typed code, using the
  `sparko_graphql` crates in a checkout of [sparko_graphql](https://github.com/bruceskingle/sparko_graphql) next to this
  one, on the branch of the same name.
- `docs/`: the user guide, an mdBook, whose `docs/help/` pages are also built into the app's help panel.

## Building and Running

- Run the app with the Dioxus CLI, `dx serve`, or `dx run`. `cargo run` does not work, as Dioxus bundles the assets.
  Give the app arguments with `--args`, e.g. `dx serve --args="--profile=test"`; the `=` is needed when the value
  starts with `-`.
- `cargo build` and `cargo test` are fine for compiling and the tests.
- The code is not formatted with rustfmt, so do not run `cargo fmt` or treat `cargo fmt --check` as a failure; match
  the formatting of the code around a change.
- Try changes with a redacted, offline profile, made with `--redact <profile> <new profile>`, see
  [docs/Redaction.md](docs/Redaction.md), or with `--home=<an empty directory>` to start afresh, rather than with a
  real account. Never put an account's details, API keys or passwords in a commit, an issue or a
  log you share.

## Branches

Each release is developed on a `vX.Y.Z` branch, in both this repo and sparko_graphql, which is squash merged to `main`
when it is released, see [docs/Release.md](docs/Release.md). Base a pull request on the release branch being developed,
not on `main`.

## Conventions

- **GraphQL nullability.** Do not put `@ms_required_by_default` on new queries in `graphql/octopus/`. Let the
  generated types follow the schema, so a field the schema allows to be null is an `Option`, and show `None` sensibly,
  e.g. blank, rather than unwrapping it. Octopus does return nulls the older queries assumed it would not.
- **Errors.** Format an error with `{error:#}`, its chain of causes, not `{error:?}`, which includes a stack trace.
- **Help.** A change to what a screen shows or does is made to its page in `docs/help/` too. A new page is added to
  `PAGES` in `src/help.rs`, see [docs/WritingHelp.md](docs/WritingHelp.md).
- **Comments** are doc comments in full sentences, saying what a thing is for and why, as the surrounding code's are.
