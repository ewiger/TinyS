//! Multi-file module discovery.
//!
//! TinyS has no `mod` keyword. A package's module tree is derived entirely from
//! the files under `src/`, and the compiler writes the `mod` declarations:
//!
//! ```text
//! src/app.sn              the file you build — the crate root
//! src/models.sn           → crate::models
//! src/models_test.sn      → crate::models_test, declared #[cfg(test)]
//! src/services/mod.sn     → crate::services
//! src/services/db.sn      → crate::services::db
//! ```
//!
//! Discovery is on only when the package has a `src/` directory *and* the file
//! being built lives inside it. Anything else is a single-file program, which is
//! how the flat `examples/` directory keeps working.

use std::path::{Path, PathBuf};

use crate::codegen::ChildModule;
use crate::manifest::Manifest;

/// One module in the derived tree.
#[derive(Debug, Clone)]
pub struct Module {
    /// The Rust identifier this module is declared as.
    pub name: String,
    /// `<name>.sn`, or a directory's `mod.sn`. `None` for a directory that has
    /// no `mod.sn` — the module exists only to hold its children.
    pub source: Option<PathBuf>,
    /// Named `*_test.sn`, so the declaration is `#[cfg(test)]`.
    pub is_test: bool,
    pub children: Vec<Module>,
}

/// A generated Rust file, positioned relative to the scratch package's `src/`.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// e.g. `main.rs`, `models.rs`, `services/mod.rs`.
    pub path: PathBuf,
    /// The `.sn` file it came from, if any.
    pub source: Option<PathBuf>,
    pub rust: String,
}

/// Every Rust file backing one `tinys build`, crate root first.
#[derive(Debug, Clone)]
pub struct Sources {
    pub files: Vec<GeneratedFile>,
}

impl Sources {
    /// Compile `entry` and, when it belongs to a package's `src/`, every module
    /// beside it.
    pub fn compile(entry: &Path, manifest: Option<&Manifest>) -> Result<Sources, String> {
        let tree = discover(entry, manifest)?;
        let children: Vec<ChildModule> = tree.iter().map(ChildModule::from).collect();

        let source = std::fs::read_to_string(entry)
            .map_err(|e| format!("error: cannot read `{}`: {}", entry.display(), e))?;
        let rust = compile_one(&source, entry, &children)?;

        let mut files = vec![GeneratedFile {
            path: PathBuf::from("main.rs"),
            source: Some(entry.to_path_buf()),
            rust,
        }];
        for module in &tree {
            emit_module(module, Path::new(""), &mut files)?;
        }
        Ok(Sources { files })
    }

    /// All generated Rust concatenated — used to decide which declared crates a
    /// program actually references.
    pub fn all_rust(&self) -> String {
        self.files
            .iter()
            .map(|f| f.rust.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render for `tinys emit-rust`. A single-file program prints exactly its
    /// own Rust; a package labels each generated file.
    pub fn render(&self) -> String {
        if self.files.len() == 1 {
            return self.files[0].rust.clone();
        }
        let mut out = String::new();
        for file in &self.files {
            out.push_str(&format!("// ---- {} ", file.path.display()));
            if let Some(src) = &file.source {
                out.push_str(&format!("(from {}) ", src.display()));
            }
            out.push_str("----\n\n");
            out.push_str(&file.rust);
            out.push('\n');
        }
        out
    }
}

impl From<&Module> for ChildModule {
    fn from(m: &Module) -> ChildModule {
        ChildModule {
            name: m.name.clone(),
            cfg_test: m.is_test,
        }
    }
}

/// Build the module tree for `entry`, or an empty tree for a single-file program.
pub fn discover(entry: &Path, manifest: Option<&Manifest>) -> Result<Vec<Module>, String> {
    let Some(manifest) = manifest else {
        return Ok(Vec::new());
    };
    let src = manifest.dir.join("src");
    if !src.is_dir() {
        return Ok(Vec::new());
    }
    // `src/` only governs files inside it; a program beside `tinys.toml` stays
    // a single file.
    let (Ok(src), Ok(entry_abs)) = (src.canonicalize(), entry.canonicalize()) else {
        return Ok(Vec::new());
    };
    if !entry_abs.starts_with(&src) {
        return Ok(Vec::new());
    }

    walk(&src, Path::new(""), &entry_abs, &manifest.exclude)
}

fn walk(dir: &Path, rel: &Path, entry: &Path, exclude: &[String]) -> Result<Vec<Module>, String> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    let listing = std::fs::read_dir(dir)
        .map_err(|e| format!("error: cannot read `{}`: {}", dir.display(), e))?;
    for item in listing {
        let path = item
            .map_err(|e| format!("error: cannot read `{}`: {}", dir.display(), e))?
            .path();
        if path.is_dir() {
            dirs.push(path);
        } else if path.extension().and_then(|e| e.to_str()) == Some("sn") {
            files.push(path);
        }
    }
    files.sort();
    dirs.sort();

    let mut modules: Vec<Module> = Vec::new();

    for path in &files {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        // A directory's own `mod.sn` belongs to the parent, not to this level.
        if stem == "mod" {
            continue;
        }
        let child_rel = rel.join(path.file_name().unwrap_or_default());
        if is_excluded(&child_rel, exclude) || same_file(path, entry) {
            continue;
        }
        modules.push(Module {
            name: module_name(stem),
            source: Some(path.clone()),
            is_test: stem.ends_with("_test"),
            children: Vec::new(),
        });
    }

    for path in &dirs {
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let child_rel = rel.join(name);
        if is_excluded(&child_rel, exclude) {
            continue;
        }
        let children = walk(path, &child_rel, entry, exclude)?;
        let mod_sn = path.join("mod.sn");
        let source = if mod_sn.is_file()
            && !is_excluded(&child_rel.join("mod.sn"), exclude)
            && !same_file(&mod_sn, entry)
        {
            Some(mod_sn)
        } else {
            None
        };
        // A directory with nothing left in it declares nothing.
        if children.is_empty() && source.is_none() {
            continue;
        }
        modules.push(Module {
            name: module_name(name),
            source,
            is_test: false,
            children,
        });
    }

    // `foo.sn` beside `foo/` would generate two modules with one name.
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    for pair in modules.windows(2) {
        if pair[0].name == pair[1].name {
            return Err(format!(
                "error: `{}` declares the module `{}` twice; rename one of the files",
                dir.display(),
                pair[0].name
            ));
        }
    }
    Ok(modules)
}

fn emit_module(
    module: &Module,
    rel_dir: &Path,
    out: &mut Vec<GeneratedFile>,
) -> Result<(), String> {
    let children: Vec<ChildModule> = module.children.iter().map(ChildModule::from).collect();
    let source = match &module.source {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("error: cannot read `{}`: {}", path.display(), e))?,
        // A directory without `mod.sn` is an empty module holding its children.
        None => String::new(),
    };
    let name_for_errors = module
        .source
        .clone()
        .unwrap_or_else(|| rel_dir.join(&module.name));
    let rust = compile_one(&source, &name_for_errors, &children)?;

    // A module with submodules must be a directory, so Rust looks for `mod.rs`.
    let path = if module.children.is_empty() {
        rel_dir.join(format!("{}.rs", module.name))
    } else {
        rel_dir.join(&module.name).join("mod.rs")
    };
    out.push(GeneratedFile {
        path,
        source: module.source.clone(),
        rust,
    });

    let child_dir = rel_dir.join(&module.name);
    for child in &module.children {
        emit_module(child, &child_dir, out)?;
    }
    Ok(())
}

fn compile_one(source: &str, file: &Path, children: &[ChildModule]) -> Result<String, String> {
    let name = file.display().to_string();
    let tokens = crate::lexer::Lexer::new(source, &name)
        .tokenize()
        .map_err(|e| e.to_string())?;
    let program = crate::parser::Parser::new(tokens, &name)
        .parse_program()
        .map_err(|e| e.to_string())?;
    Ok(crate::codegen::generate_module(&program, children))
}

/// File stems become Rust identifiers.
fn module_name(stem: &str) -> String {
    let mut name: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        name.insert(0, '_');
    }
    name
}

fn same_file(a: &Path, b: &Path) -> bool {
    match a.canonicalize() {
        Ok(a) => a == b,
        Err(_) => a == b,
    }
}

/// Match a path against `exclude` from `tinys.toml`. Patterns are relative to
/// `src/`; `*` matches within one path segment, and naming a directory excludes
/// everything under it.
fn is_excluded(rel: &Path, exclude: &[String]) -> bool {
    let path = rel.to_string_lossy().replace('\\', "/");
    exclude.iter().any(|pattern| {
        let pattern = pattern.trim_end_matches('/');
        glob_match(pattern, &path) || path.starts_with(&format!("{}/", pattern))
    })
}

/// Glob with a single `*` wildcard that does not cross `/`.
fn glob_match(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, tail)) => {
            if !text.starts_with(head) || text.len() < head.len() + tail.len() {
                return false;
            }
            let rest = &text[head.len()..];
            // The wildcard stops at a path separator.
            let stop = rest.find('/').unwrap_or(rest.len());
            (0..=stop)
                .filter(|i| rest.is_char_boundary(*i))
                .any(|i| glob_match(tail, &rest[i..]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_names_become_identifiers() {
        assert_eq!(module_name("models"), "models");
        assert_eq!(module_name("my-mod"), "my_mod");
        assert_eq!(module_name("2fast"), "_2fast");
    }

    #[test]
    fn globs_stop_at_path_separators() {
        assert!(glob_match("wip.sn", "wip.sn"));
        assert!(glob_match("*.sn", "models.sn"));
        assert!(!glob_match("*.sn", "services/models.sn"));
        assert!(glob_match("scratch/*.sn", "scratch/a.sn"));
        assert!(glob_match("*_wip.sn", "draft_wip.sn"));
        assert!(!glob_match("*_wip.sn", "draft.sn"));
    }

    #[test]
    fn excluding_a_directory_excludes_its_contents() {
        let exclude = vec!["scratch".to_string()];
        assert!(is_excluded(Path::new("scratch"), &exclude));
        assert!(is_excluded(Path::new("scratch/a.sn"), &exclude));
        assert!(is_excluded(Path::new("scratch/deep/b.sn"), &exclude));
        assert!(!is_excluded(Path::new("scratchpad.sn"), &exclude));
    }
}
