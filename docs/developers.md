# For Developers

Marco Sparko is open source, written in Rust with [Dioxus](https://dioxuslabs.com). The source code, issues and releases are on GitHub at [bruceskingle/marco-sparko](https://github.com/bruceskingle/marco-sparko).

The documentation for developers is kept with the source code, rather than in this book:

- [Writing Help](https://github.com/bruceskingle/marco-sparko/blob/main/docs/WritingHelp.md), how the help in the app and in this book is written and built in.
- [Release](https://github.com/bruceskingle/marco-sparko/blob/main/docs/Release.md), how a release is built and published.
- [The Octopus Energy plugin](https://github.com/bruceskingle/marco-sparko/blob/main/docs/octopus/index.md), notes on Octopus Energy's GraphQL API, including the queries the Octopus web site makes.
- [GraphQL Playground HowTo](https://github.com/bruceskingle/marco-sparko/blob/main/docs/howto/GraphQL.md), an introduction to GraphQL and to making queries interactively.
- [Chrome Browser Inspector HowTo](https://github.com/bruceskingle/marco-sparko/blob/main/docs/howto/ChromeInspector.md), watching the API calls a web page makes.

This book is built from the `docs` directory with [mdBook](https://rust-lang.github.io/mdBook/), see `book.toml`, and published to GitHub Pages whenever `main` changes.
