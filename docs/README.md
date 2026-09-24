# Documentation

This directory is the source for the published SorobanLabs Analyzer
documentation book, built with [mdBook](https://rust-lang.github.io/mdBook/).

- `book.toml` — book configuration.
- `src/` — the actual page content; `src/SUMMARY.md` defines the table
  of contents.

## Building locally

```
cargo install mdbook --locked
mdbook build docs
```

The rendered site is written to `docs/book/`. To preview it with live
reload while editing:

```
mdbook serve docs
```

## Publishing

The book is published to GitHub Pages by
`.github/workflows/docs.yml` on every push to `main`, using the
official `actions/configure-pages`, `actions/upload-pages-artifact`,
and `actions/deploy-pages` actions. The live site is linked from the
top-level [README.md](../README.md).
