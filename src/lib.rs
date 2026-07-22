//! TinyS — a small statically typed language with Python-shaped syntax and
//! Rust-oriented semantics.
//!
//! The library exposes the compilation pipeline:
//!
//! ```text
//! .sn source → lexer → parser → AST → Rust code generator → generated .rs
//! ```
//!
//! The generated Rust is then handed to Cargo by the `tinys` CLI, which wraps it
//! in a scratch Cargo package whose dependencies come from `tinys.toml`.

pub mod ast;
pub mod codegen;
pub mod diagnostics;
pub mod lexer;
pub mod manifest;
pub mod modules;
pub mod parser;

pub use diagnostics::TinysError;

/// Compile TinyS source text into generated Rust source text.
///
/// `filename` is only used for diagnostics.
pub fn compile_to_rust(source: &str, filename: &str) -> Result<String, TinysError> {
    let tokens = lexer::Lexer::new(source, filename).tokenize()?;
    let program = parser::Parser::new(tokens, filename).parse_program()?;
    Ok(codegen::generate(&program))
}
