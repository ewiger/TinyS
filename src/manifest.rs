//! Reading `tinys.toml`, the TinyS package manifest.
//!
//! `tinys.toml` mirrors `Cargo.toml`: a `[package]` table plus a `[dependencies]`
//! table whose entries are copied into the `Cargo.toml` that backs a build.
//!
//! ```toml
//! [package]
//! name = "example"
//! version = "0.1.0"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```
//!
//! Only the subset TinyS needs is understood: `[package]` keys are read as plain
//! strings, and dependency values are kept as raw TOML text so Cargo — not this
//! module — decides what they mean. `[profile.*]` and `[patch.*]` sections are
//! passed through verbatim; anything else is reported through [`Manifest::ignored`].

use std::path::{Path, PathBuf};

/// The `[package]` table.
#[derive(Debug, Default, Clone)]
pub struct Package {
    pub name: Option<String>,
    pub version: Option<String>,
    /// Rust edition for the generated crate.
    pub edition: Option<String>,
}

/// A dependency value, kept as the raw TOML the user wrote.
#[derive(Debug, Clone)]
pub enum DepBody {
    /// `serde_json = "1"` → `"1"`, or an inline table `{ version = "1" }`.
    Inline(String),
    /// A `[dependencies.<name>]` sub-table, as `key = value` lines.
    Table(Vec<String>),
}

/// One entry of `[dependencies]`.
#[derive(Debug, Clone)]
pub struct Dependency {
    /// The key as written; also the identifier the generated Rust refers to.
    pub name: String,
    pub body: DepBody,
}

/// A `[profile.*]` / `[patch.*]` section, forwarded to Cargo unchanged.
#[derive(Debug, Clone)]
pub struct Section {
    pub header: String,
    pub lines: Vec<String>,
}

/// A parsed `tinys.toml`.
#[derive(Debug, Clone)]
pub struct Manifest {
    /// The manifest file itself.
    pub path: PathBuf,
    /// Directory holding the manifest — the package root.
    pub dir: PathBuf,
    pub package: Package,
    pub dependencies: Vec<Dependency>,
    pub passthrough: Vec<Section>,
    /// Section headers TinyS does not understand yet.
    pub ignored: Vec<String>,
}

impl Manifest {
    /// Look for the package manifest governing `source_file` by walking up from
    /// its directory, mirroring how Cargo finds `Cargo.toml`.
    pub fn discover(source_file: &Path) -> Result<Option<Manifest>, String> {
        let start = source_file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

        let mut dir = Some(start.as_path());
        while let Some(d) = dir {
            let candidate = d.join("tinys.toml");
            if candidate.is_file() {
                return Manifest::parse_file(&candidate).map(Some);
            }
            dir = d.parent();
        }
        Ok(None)
    }

    pub fn parse_file(path: &Path) -> Result<Manifest, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read `{}`: {}", path.display(), e))?;
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        Manifest::parse(&text, path, &dir)
    }

    fn parse(text: &str, path: &Path, dir: &Path) -> Result<Manifest, String> {
        let mut manifest = Manifest {
            path: path.to_path_buf(),
            dir: dir.to_path_buf(),
            package: Package::default(),
            dependencies: Vec::new(),
            passthrough: Vec::new(),
            ignored: Vec::new(),
        };

        let lines: Vec<&str> = text.lines().collect();
        let mut section = String::new();
        let mut i = 0;
        while i < lines.len() {
            let raw = lines[i];
            i += 1;
            let line = strip_comment(raw).trim();
            if line.is_empty() {
                continue;
            }

            if let Some(header) = table_header(line) {
                section = header.to_string();
                let known = section == "package"
                    || section == "dependencies"
                    || section.starts_with("dependencies.")
                    || section == "profile"
                    || section.starts_with("profile.")
                    || section == "patch"
                    || section.starts_with("patch.");
                if !known && !manifest.ignored.contains(&section) {
                    manifest.ignored.push(section.clone());
                }
                if section.starts_with("profile") || section.starts_with("patch") {
                    manifest.passthrough.push(Section {
                        header: section.clone(),
                        lines: Vec::new(),
                    });
                }
                if let Some(name) = section.strip_prefix("dependencies.") {
                    manifest.dependencies.push(Dependency {
                        name: name.trim_matches('"').to_string(),
                        body: DepBody::Table(Vec::new()),
                    });
                }
                continue;
            }

            // A `key = value` line. Inline tables and arrays may wrap, so keep
            // pulling lines in until the brackets balance.
            let mut value_text = line.to_string();
            while !balanced(&value_text) && i < lines.len() {
                let next = strip_comment(lines[i]).trim_end();
                i += 1;
                value_text.push('\n');
                value_text.push_str(next);
            }

            let Some((key, value)) = split_key_value(&value_text) else {
                return Err(format!(
                    "{}: cannot parse `{}` (expected `key = value`)",
                    path.display(),
                    line
                ));
            };

            match section.as_str() {
                "package" => match key.as_str() {
                    "name" => manifest.package.name = Some(unquote(&value)),
                    "version" => manifest.package.version = Some(unquote(&value)),
                    "edition" => manifest.package.edition = Some(unquote(&value)),
                    _ => {}
                },
                "dependencies" => manifest.dependencies.push(Dependency {
                    name: key,
                    body: DepBody::Inline(value),
                }),
                s if s.starts_with("dependencies.") => {
                    if let Some(Dependency {
                        body: DepBody::Table(body),
                        ..
                    }) = manifest.dependencies.last_mut()
                    {
                        body.push(format!("{} = {}", key, value));
                    }
                }
                s if s.starts_with("profile") || s.starts_with("patch") => {
                    if let Some(sec) = manifest.passthrough.last_mut() {
                        sec.lines.push(format!("{} = {}", key, value));
                    }
                }
                _ => {}
            }
        }

        manifest.absolutize_paths();
        Ok(manifest)
    }

    /// Dependencies are declared relative to `tinys.toml`, but the generated
    /// `Cargo.toml` lives under `target/`, so `path = "..."` must be absolute.
    fn absolutize_paths(&mut self) {
        let dir = self.dir.clone();
        for dep in &mut self.dependencies {
            match &mut dep.body {
                DepBody::Inline(value) => *value = rewrite_paths(value, &dir),
                DepBody::Table(lines) => {
                    for line in lines {
                        *line = rewrite_paths(line, &dir);
                    }
                }
            }
        }
        for section in &mut self.passthrough {
            for line in &mut section.lines {
                *line = rewrite_paths(line, &dir);
            }
        }
    }

    /// Render the dependency and passthrough tables for a generated `Cargo.toml`,
    /// keeping only the dependencies `keep` accepts.
    pub fn cargo_sections(&self, keep: impl Fn(&str) -> bool) -> String {
        let kept: Vec<&Dependency> = self.dependencies.iter().filter(|d| keep(&d.name)).collect();

        let mut out = String::new();
        let inline: Vec<&Dependency> = kept
            .iter()
            .copied()
            .filter(|d| matches!(d.body, DepBody::Inline(_)))
            .collect();
        if !inline.is_empty() {
            out.push_str("\n[dependencies]\n");
            for dep in inline {
                if let DepBody::Inline(value) = &dep.body {
                    out.push_str(&format!("{} = {}\n", dep.name, value));
                }
            }
        }
        for dep in &kept {
            if let DepBody::Table(lines) = &dep.body {
                out.push_str(&format!("\n[dependencies.{}]\n", dep.name));
                for line in lines {
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        for section in &self.passthrough {
            out.push_str(&format!("\n[{}]\n", section.header));
            for line in &section.lines {
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    }
}

/// Does the generated Rust refer to crate `dep`?
///
/// A TinyS program imports every crate it uses (`from rust.serde import ...`,
/// `import rust.serde_json as json`), so the crate identifier always appears in
/// the generated source. Dependencies that do not appear are left out of the
/// generated `Cargo.toml`, which keeps single-file builds fast — and offline —
/// even when the package declares crates this program never touches.
pub fn references_crate(rust: &str, dep: &str) -> bool {
    let ident = dep.replace('-', "_");
    if ident.is_empty() {
        return false;
    }
    let bytes = rust.as_bytes();
    let mut from = 0;
    while let Some(found) = rust[from..].find(&ident) {
        let start = from + found;
        let end = start + ident.len();
        let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

/// `[section]` → `section`; `[[array]]` and plain values → `None`.
fn table_header(line: &str) -> Option<&str> {
    let rest = line.strip_prefix('[')?;
    if rest.starts_with('[') {
        return None;
    }
    let end = rest.rfind(']')?;
    Some(rest[..end].trim())
}

fn split_key_value(line: &str) -> Option<(String, String)> {
    let eq = index_outside_string(line, '=')?;
    let key = line[..eq].trim().trim_matches('"').to_string();
    let value = line[eq + 1..].trim().to_string();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key, value))
}

fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

/// Drop a trailing `# comment`, ignoring `#` inside strings.
fn strip_comment(line: &str) -> &str {
    match index_outside_string(line, '#') {
        Some(i) => &line[..i],
        None => line,
    }
}

fn index_outside_string(line: &str, needle: char) -> Option<usize> {
    let mut in_str = false;
    let mut quote = '"';
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' && quote == '"' {
                escaped = true;
            } else if c == quote {
                in_str = false;
            }
            continue;
        }
        if c == needle {
            return Some(i);
        }
        if c == '"' || c == '\'' {
            in_str = true;
            quote = c;
        }
    }
    None
}

/// Are all `{`/`[` in `text` closed? Used to join wrapped inline tables.
fn balanced(text: &str) -> bool {
    let mut depth = 0i32;
    let mut in_str = false;
    let mut quote = '"';
    let mut escaped = false;
    for c in text.chars() {
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' && quote == '"' {
                escaped = true;
            } else if c == quote {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_str = true;
                quote = c;
            }
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            _ => {}
        }
    }
    depth <= 0
}

/// Make every `path = "…"` in a dependency value absolute against `base`.
fn rewrite_paths(value: &str, base: &Path) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(at) = find_key(rest, "path") {
        let (head, tail) = rest.split_at(at);
        out.push_str(head);
        // `path` … `=` … `"value"`
        let after_key = &tail[4..];
        let Some(eq) = after_key.find('=') else {
            out.push_str(tail);
            return out;
        };
        let after_eq = &after_key[eq + 1..];
        let trimmed = after_eq.trim_start();
        let pad = after_eq.len() - trimmed.len();
        let Some(quote) = trimmed.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            out.push_str(&tail[..4 + eq + 1]);
            rest = after_eq;
            continue;
        };
        let Some(close) = trimmed[1..].find(quote) else {
            out.push_str(tail);
            return out;
        };
        let literal = &trimmed[1..1 + close];
        let resolved = if Path::new(literal).is_absolute() {
            literal.to_string()
        } else {
            let joined = base.join(literal);
            joined
                .canonicalize()
                .unwrap_or(joined)
                .display()
                .to_string()
                .replace('\\', "\\\\")
        };
        out.push_str(&tail[..4 + eq + 1]);
        out.push_str(&" ".repeat(pad));
        out.push(quote);
        out.push_str(&resolved);
        out.push(quote);
        rest = &trimmed[1 + close + 1..];
    }
    out.push_str(rest);
    out
}

/// Find `key` used as a bare TOML key (not part of a longer identifier).
fn find_key(text: &str, key: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(found) = text[from..].find(key) {
        let start = from + found;
        let end = start + key.len();
        let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            return Some(start);
        }
        from = start + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Manifest {
        Manifest::parse(text, Path::new("/pkg/tinys.toml"), Path::new("/pkg")).unwrap()
    }

    #[test]
    fn reads_package_and_dependencies() {
        let m = parse(
            r#"
[package]
name = "example"      # trailing comment
version = "0.1.0"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        );
        assert_eq!(m.package.name.as_deref(), Some("example"));
        assert_eq!(m.package.version.as_deref(), Some("0.1.0"));
        assert_eq!(m.dependencies.len(), 2);
        assert_eq!(m.dependencies[0].name, "serde");
        let rendered = m.cargo_sections(|_| true);
        assert!(rendered.contains("serde = { version = \"1\", features = [\"derive\"] }"));
        assert!(rendered.contains("serde_json = \"1\""));
    }

    #[test]
    fn reads_dependency_sub_tables() {
        let m = parse(
            r#"
[dependencies.regex]
version = "1"
default-features = false
"#,
        );
        assert_eq!(m.dependencies.len(), 1);
        let rendered = m.cargo_sections(|_| true);
        assert!(rendered.contains("[dependencies.regex]"));
        assert!(rendered.contains("default-features = false"));
    }

    #[test]
    fn joins_wrapped_inline_tables() {
        let m = parse(
            "[dependencies]\ntokio = { version = \"1\",\n  features = [\"full\"] }\nlog = \"0.4\"\n",
        );
        assert_eq!(m.dependencies.len(), 2);
        assert_eq!(m.dependencies[1].name, "log");
    }

    #[test]
    fn keeps_only_referenced_dependencies() {
        let m = parse("[dependencies]\nserde_json = \"1\"\nregex = \"1\"\n");
        let rust = "use serde_json;\nfn main() {}\n";
        let rendered = m.cargo_sections(|name| references_crate(rust, name));
        assert!(rendered.contains("serde_json"));
        assert!(!rendered.contains("regex"));
    }

    #[test]
    fn crate_references_match_whole_identifiers() {
        assert!(references_crate("use serde::Deserialize;", "serde"));
        assert!(!references_crate("use serde_json::from_str;", "serde"));
        assert!(references_crate("use serde_json::from_str;", "serde_json"));
        assert!(references_crate("#[tokio::main]", "tokio"));
        // Cargo names may use `-`; the Rust identifier uses `_`.
        assert!(references_crate(
            "use pretty_env_logger;",
            "pretty-env-logger"
        ));
    }

    #[test]
    fn dependency_paths_become_absolute() {
        let m = parse("[dependencies]\nlocal = { path = \"../vendor/local\" }\n");
        let rendered = m.cargo_sections(|_| true);
        assert!(
            rendered.contains("path = \"/vendor/local\"")
                || rendered.contains("/pkg/../vendor/local")
        );
        assert!(!rendered.contains("\"../vendor/local\""));
    }

    #[test]
    fn records_unsupported_sections() {
        let m = parse("[workspace]\nmembers = []\n");
        assert_eq!(m.ignored, vec!["workspace".to_string()]);
    }
}
