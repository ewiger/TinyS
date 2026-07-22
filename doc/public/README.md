# TinyS documentation site

This directory holds the public **TinyS language manual**, built with
[MkDocs](https://www.mkdocs.org/) and the
[Material for MkDocs](https://squidfunk.github.io/mkdocs-material/) theme and
published to GitHub Pages at **<https://ewiger.github.io/TinyS/>**.

```text
doc/public/
├── mkdocs.yml          # site configuration + navigation
├── requirements.txt    # docs toolchain (mkdocs-material)
├── docs/               # the Markdown sources for every page
│   ├── index.md
│   ├── getting-started/
│   ├── guide/
│   ├── advanced/
│   ├── examples/
│   ├── reference/
│   ├── about/
│   └── stylesheets/extra.css
└── site/               # build output (git-ignored)
```

## Build and preview locally

```bash
# From the repository root
python3 -m venv .venv && source .venv/bin/activate      # optional
pip install -r doc/public/requirements.txt

# Live-reloading preview at http://127.0.0.1:8000
mkdocs serve --config-file doc/public/mkdocs.yml

# One-off production build into doc/public/site/
mkdocs build --strict --config-file doc/public/mkdocs.yml
```

`--strict` turns warnings (broken links, pages missing from the nav) into
errors — the CI build uses it, so run it before pushing.

## How it gets published

Every push to `main` that touches `doc/public/**` triggers the
[`docs.yml`](../../.github/workflows/docs.yml) GitHub Actions workflow, which
builds the site and deploys it to GitHub Pages. See
[`about/contributing`](docs/about/contributing.md) for the one-time repository
setup (Settings → Pages → *Build and deployment* → *GitHub Actions*).
