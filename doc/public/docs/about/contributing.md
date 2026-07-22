# Contributing

TinyS is still in the language-design stage, so contributions of ideas are as
valuable as code.

## Useful contributions

- syntax proposals with concrete examples;
- ambiguity analysis;
- Rust translation examples;
- parser prototypes;
- compiler architecture experiments;
- diagnostic mapping;
- formatter design;
- ownership and lifetime edge cases;
- interoperability tests against real Rust crates.

Proposals should include **both** the TinyS source and the expected generated
Rust — that pairing is what keeps the design honest.

## Working on the compiler

```bash
git clone https://github.com/ewiger/TinyS.git
cd TinyS
cargo build            # build the tinys compiler
cargo test             # lexer, codegen, and end-to-end example tests
cargo run -- run examples/hello.sn
```

The end-to-end tests drive the `tinys` binary to compile the `.sn` examples with
`rustc`; they are skipped automatically when `rustc` is unavailable.

## Working on these docs

The manual is built with [MkDocs](https://www.mkdocs.org/) + Material and lives in
[`doc/public/`](https://github.com/ewiger/TinyS/tree/main/doc/public).

```bash
pip install -r doc/public/requirements.txt

# Live preview at http://127.0.0.1:8000
mkdocs serve --config-file doc/public/mkdocs.yml

# Production build (the CI check)
mkdocs build --strict --config-file doc/public/mkdocs.yml
```

`--strict` turns warnings (broken links, pages missing from the nav) into errors,
so run it before opening a pull request. Every page must appear in the `nav:`
section of `mkdocs.yml`.

### Conventions in the docs

- TinyS code blocks are fenced as ```` ```python ```` — TinyS is Python-shaped, so
  Python highlighting is the closest visual match. Generated Rust uses
  ```` ```rust ````.
- When a page describes a feature that does not yet compile, mark it with an
  admonition and link to [Language status](status.md).

## How the docs get published

Publishing is automated. On every push to `main` that touches `doc/public/**`, the
[`docs.yml`](https://github.com/ewiger/TinyS/blob/main/.github/workflows/docs.yml)
GitHub Actions workflow builds the site with `--strict` and deploys it to GitHub
Pages.

### One-time repository setup

Whoever administers the repository needs to enable Pages once:

1. Go to **Settings → Pages**.
2. Under **Build and deployment → Source**, choose **GitHub Actions**.

After that, pushes to `main` publish automatically to
**<https://ewiger.github.io/TinyS/>**. You can also trigger a build manually from
the **Actions** tab (the workflow allows `workflow_dispatch`).

## License

No license has been selected yet. Until one is added, the repository should be
considered source-available but not automatically licensed for redistribution or
reuse.
