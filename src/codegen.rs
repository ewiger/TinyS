//! Rust source generator for TinyS.
//!
//! Walks the [`ast::Program`] and produces readable, deterministic Rust source.
//! A light pre-scan collects struct field types, function signatures, and import
//! aliases so a few surface conveniences (owned-string coercion, `::` vs `.`
//! member access, macro names) can be resolved without a full type checker.

use crate::ast::*;
use std::collections::{HashMap, HashSet};

/// A `mod` declaration synthesized from the file tree.
///
/// TinyS has no `mod` keyword: submodules are derived from the `.sn` files next
/// to their parent, and the compiler writes these declarations itself.
#[derive(Debug, Clone)]
pub struct ChildModule {
    pub name: String,
    /// Declared `#[cfg(test)]`, from a `_test.sn` file.
    pub cfg_test: bool,
}

pub fn generate(program: &Program) -> String {
    generate_module(program, &[])
}

/// Generate a module that declares `children` as submodules.
pub fn generate_module(program: &Program, children: &[ChildModule]) -> String {
    let mut cg = Codegen::new();
    cg.prescan(program);
    cg.emit_program(program, children)
}

struct Sig {
    params: Vec<Type>,
}

struct Codegen {
    struct_fields: HashMap<String, Vec<Field>>,
    fn_sigs: HashMap<String, Sig>,
    /// Import alias (e.g. `json`) → real path base (e.g. `serde_json`).
    module_alias: HashMap<String, String>,
    /// Identifiers that name a module/crate path (used for `::` member access).
    modules: HashSet<String>,
    /// TinyS macro-callable name → Rust macro name (without `!`).
    macro_names: HashMap<String, String>,
    /// Lexical scope stack of declared value bindings (for `let` vs reassign).
    scopes: Vec<HashSet<String>>,
    cur_ret: Option<Type>,
}

impl Codegen {
    fn new() -> Self {
        let mut macro_names = HashMap::new();
        for (tinys, rust) in [
            ("print", "println"),
            ("println", "println"),
            ("eprint", "eprintln"),
            ("format", "format"),
            ("debug", "dbg"),
            ("assert", "assert"),
            ("assert_eq", "assert_eq"),
            ("panic", "panic"),
            ("vec", "vec"),
        ] {
            macro_names.insert(tinys.to_string(), rust.to_string());
        }
        Codegen {
            struct_fields: HashMap::new(),
            fn_sigs: HashMap::new(),
            module_alias: HashMap::new(),
            modules: HashSet::new(),
            macro_names,
            scopes: vec![HashSet::new()],
            cur_ret: None,
        }
    }

    // ---- pre-scan -------------------------------------------------------------

    fn prescan(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Struct(s) => {
                    self.struct_fields.insert(s.name.clone(), s.fields.clone());
                }
                Item::Function(f) => {
                    self.fn_sigs.insert(
                        f.name.clone(),
                        Sig {
                            params: f.params.iter().map(|p| p.ty.clone()).collect(),
                        },
                    );
                }
                Item::Use(u) => self.scan_use(u),
                _ => {}
            }
        }
    }

    fn scan_use(&mut self, u: &Use) {
        if u.is_macro {
            // `from macro import ...` and `from macro.std import ...` name prelude
            // and std macros, which are callable unqualified. Any other root is a
            // crate namespace (`from macro.serde_json import json`), so the call
            // site is emitted path-qualified: `serde_json::json!(...)`.
            let crate_path = match u.path.first().map(String::as_str) {
                None | Some("std") => None,
                Some(_) => Some(u.path.join("::")),
            };
            for n in &u.names {
                let callable = n.alias.clone().unwrap_or_else(|| n.name.clone());
                let rust = match &crate_path {
                    Some(base) => format!("{}::{}", base, n.name),
                    None => self
                        .macro_names
                        .get(&n.name)
                        .cloned()
                        .unwrap_or_else(|| n.name.clone()),
                };
                self.macro_names.insert(callable, rust);
            }
            return;
        }
        let prefix = if u.is_rust {
            String::new()
        } else {
            "crate::".to_string()
        };
        if let Some(alias) = &u.alias {
            let full = format!("{}{}", prefix, u.path.join("::"));
            self.module_alias.insert(alias.clone(), full);
            self.modules.insert(alias.clone());
        }
        // `import rust.serde_json` (no alias) → the last segment is a usable module.
        if u.alias.is_none() && u.names.is_empty() {
            if let Some(last) = u.path.last() {
                self.modules.insert(last.clone());
            }
        }
    }

    // ---- scope helpers --------------------------------------------------------

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn declare(&mut self, name: &str) {
        self.scopes.last_mut().unwrap().insert(name.to_string());
    }
    fn declared(&self, name: &str) -> bool {
        self.scopes.iter().any(|s| s.contains(name))
    }

    // ---- program & items ------------------------------------------------------

    fn emit_program(&mut self, program: &Program, children: &[ChildModule]) -> String {
        let mut use_lines: Vec<String> = Vec::new();
        let mut bodies: Vec<String> = Vec::new();

        for item in &program.items {
            match item {
                Item::Use(u) => use_lines.extend(self.gen_use(u)),
                Item::Function(f) => bodies.push(self.gen_function(f, false)),
                Item::Struct(s) => bodies.push(self.gen_struct(s)),
                Item::Enum(e) => bodies.push(self.gen_enum(e)),
                Item::Trait(t) => bodies.push(self.gen_trait(t)),
                Item::Impl(i) => bodies.push(self.gen_impl(i)),
            }
        }

        let body = bodies.join("\n\n");

        // Auto-import standard collections when referenced but not imported.
        let mut auto: Vec<String> = Vec::new();
        let joined_uses = use_lines.join("\n");
        if body.contains("HashMap") && !joined_uses.contains("HashMap") {
            auto.push("use std::collections::HashMap;".to_string());
        }
        if body.contains("HashSet") && !joined_uses.contains("HashSet") {
            auto.push("use std::collections::HashSet;".to_string());
        }

        let mut out = String::new();
        out.push_str(
            "#![allow(dead_code, unused_variables, unused_imports, unused_mut, unused_parens, non_snake_case, clippy::all)]\n",
        );
        if !children.is_empty() {
            out.push('\n');
            for child in children {
                if child.cfg_test {
                    out.push_str("#[cfg(test)]\n");
                }
                out.push_str(&format!("pub mod {};\n", child.name));
            }
        }
        if !use_lines.is_empty() || !auto.is_empty() {
            out.push('\n');
            for l in &auto {
                out.push_str(l);
                out.push('\n');
            }
            for l in &use_lines {
                out.push_str(l);
                out.push('\n');
            }
        }
        out.push('\n');
        out.push_str(&body);
        out.push('\n');
        out
    }

    fn gen_use(&self, u: &Use) -> Vec<String> {
        if u.is_macro {
            return Vec::new();
        }
        let prefix = if u.is_rust { "" } else { "crate::" };
        let base = format!("{}{}", prefix, u.path.join("::"));

        if !u.names.is_empty() {
            let items: Vec<String> = u
                .names
                .iter()
                .map(|n| match &n.alias {
                    Some(a) => format!("{} as {}", n.name, a),
                    None => n.name.clone(),
                })
                .collect();
            if items.len() == 1 {
                return vec![format!("use {}::{};", base, items[0])];
            }
            return vec![format!("use {}::{{{}}};", base, items.join(", "))];
        }
        vec![format!("use {};", base)]
    }

    fn gen_attrs(&self, attrs: &[Attr]) -> String {
        attrs
            .iter()
            .map(|a| format!("#[{}]\n", a))
            .collect::<String>()
    }

    fn gen_function(&mut self, f: &Function, _is_method: bool) -> String {
        let attrs = self.gen_attrs(&f.attrs);
        let vis = if f.is_pub { "pub " } else { "" };
        let asy = if f.is_async { "async " } else { "" };
        let uns = if f.is_unsafe { "unsafe " } else { "" };
        let generics = self.gen_generics(&f.generics);

        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| {
                if p.name == "self" {
                    self.gen_receiver(&p.ty)
                } else {
                    format!("{}: {}", p.name, self.gen_type(&p.ty))
                }
            })
            .collect();

        let ret = match &f.ret {
            Type::Unit => String::new(),
            other => format!(" -> {}", self.gen_type(other)),
        };

        let sig = format!(
            "{}{}{}{}fn {}{}({}){}",
            attrs,
            vis,
            asy,
            uns,
            f.name,
            generics,
            params.join(", "),
            ret
        );

        match &f.body {
            None => format!("{};", sig),
            Some(body) => {
                self.push_scope();
                for p in &f.params {
                    self.declare(&p.name);
                }
                let prev_ret = self.cur_ret.take();
                self.cur_ret = match &f.ret {
                    Type::Unit => None,
                    other => Some(other.clone()),
                };
                // Only a value-returning function treats its final expression as
                // an implicit return; a `void` function keeps every statement a
                // statement (so e.g. a trailing `dbg!(x)` yields `()`).
                let is_value = self.cur_ret.is_some();
                let tail = self.cur_ret.clone();
                let block = self.gen_block(body, is_value, tail.as_ref());
                self.cur_ret = prev_ret;
                self.pop_scope();
                format!("{} {}", sig, block)
            }
        }
    }

    fn gen_receiver(&self, ty: &Type) -> String {
        match ty {
            Type::Path { segments, .. } if segments == &["Self".to_string()] => "self".to_string(),
            Type::Ref {
                mutable,
                lifetime,
                inner,
            } if matches!(&**inner, Type::Path { segments, .. } if segments == &["Self".to_string()]) =>
            {
                let lt = lifetime
                    .as_ref()
                    .map(|l| format!("'{} ", l))
                    .unwrap_or_default();
                let m = if *mutable { "mut " } else { "" };
                format!("&{}{}self", lt, m)
            }
            other => format!("self: {}", self.gen_type(other)),
        }
    }

    fn gen_struct(&self, s: &Struct) -> String {
        let attrs = self.gen_attrs(&s.attrs);
        let vis = if s.is_pub { "pub " } else { "" };
        let generics = self.gen_generics(&s.generics);
        if s.fields.is_empty() {
            return format!("{}{}struct {}{};", attrs, vis, s.name, generics);
        }
        let fields: Vec<String> = s
            .fields
            .iter()
            .map(|f| {
                let fv = if f.is_pub { "pub " } else { "" };
                format!("{}{}: {},", fv, f.name, self.gen_type(&f.ty))
            })
            .collect();
        format!(
            "{}{}struct {}{} {{\n{}\n}}",
            attrs,
            vis,
            s.name,
            generics,
            indent(&fields.join("\n"), 1)
        )
    }

    fn gen_enum(&self, e: &Enum) -> String {
        let attrs = self.gen_attrs(&e.attrs);
        let vis = if e.is_pub { "pub " } else { "" };
        let generics = self.gen_generics(&e.generics);
        let variants: Vec<String> = e
            .variants
            .iter()
            .map(|v| {
                if v.payload.is_empty() {
                    format!("{},", v.name)
                } else {
                    let tys: Vec<String> = v.payload.iter().map(|t| self.gen_type(t)).collect();
                    format!("{}({}),", v.name, tys.join(", "))
                }
            })
            .collect();
        format!(
            "{}{}enum {}{} {{\n{}\n}}",
            attrs,
            vis,
            e.name,
            generics,
            indent(&variants.join("\n"), 1)
        )
    }

    fn gen_trait(&mut self, t: &Trait) -> String {
        let attrs = self.gen_attrs(&t.attrs);
        let vis = if t.is_pub { "pub " } else { "" };
        let generics = self.gen_generics(&t.generics);
        let methods: Vec<String> = t
            .methods
            .iter()
            .map(|m| self.gen_function(m, true))
            .collect();
        format!(
            "{}{}trait {}{} {{\n{}\n}}",
            attrs,
            vis,
            t.name,
            generics,
            indent(&methods.join("\n\n"), 1)
        )
    }

    fn gen_impl(&mut self, i: &Impl) -> String {
        let generics = self.gen_generics(&i.generics);
        let head = match &i.trait_name {
            Some(tr) => format!(
                "impl{} {} for {}",
                generics,
                self.gen_type(tr),
                self.gen_type(&i.ty)
            ),
            None => format!("impl{} {}", generics, self.gen_type(&i.ty)),
        };
        let methods: Vec<String> = i
            .methods
            .iter()
            .map(|m| self.gen_function(m, true))
            .collect();
        format!("{} {{\n{}\n}}", head, indent(&methods.join("\n\n"), 1))
    }

    fn gen_generics(&self, params: &[GenericParam]) -> String {
        if params.is_empty() {
            return String::new();
        }
        let mut lifetimes = Vec::new();
        let mut types = Vec::new();
        for p in params {
            match p {
                GenericParam::Lifetime(l) => lifetimes.push(format!("'{}", l)),
                GenericParam::Type { name, bounds } => {
                    if bounds.is_empty() {
                        types.push(name.clone());
                    } else {
                        types.push(format!("{}: {}", name, bounds.join(" + ")));
                    }
                }
            }
        }
        let all: Vec<String> = lifetimes.into_iter().chain(types).collect();
        format!("<{}>", all.join(", "))
    }

    // ---- types ----------------------------------------------------------------

    fn gen_type(&self, ty: &Type) -> String {
        match ty {
            Type::Unit => "()".to_string(),
            Type::Path { segments, args } => {
                if segments.len() == 1 && segments[0].starts_with('\'') {
                    return segments[0].clone();
                }
                let base = if segments.len() == 1 {
                    map_type_name(&segments[0])
                } else {
                    let mut segs = segments.clone();
                    if let Some(real) = self.module_alias.get(&segs[0]) {
                        segs[0] = real.clone();
                    }
                    segs.join("::")
                };
                if args.is_empty() {
                    base
                } else {
                    let a: Vec<String> = args.iter().map(|t| self.gen_type(t)).collect();
                    format!("{}<{}>", base, a.join(", "))
                }
            }
            Type::Ref {
                mutable,
                lifetime,
                inner,
            } => {
                let lt = lifetime
                    .as_ref()
                    .map(|l| format!("'{} ", l))
                    .unwrap_or_default();
                let m = if *mutable { "mut " } else { "" };
                // `ref str`/`ref slice[T]` borrow the unsized form, not the owned one.
                let inner_code = match &**inner {
                    Type::Path { segments, args }
                        if args.is_empty() && segments == &["str".to_string()] =>
                    {
                        "str".to_string()
                    }
                    other => self.gen_type(other),
                };
                format!("&{}{}{}", lt, m, inner_code)
            }
            Type::Tuple(elems) => {
                let e: Vec<String> = elems.iter().map(|t| self.gen_type(t)).collect();
                format!("({})", e.join(", "))
            }
            Type::Array { elem, len } => {
                format!("[{}; {}]", self.gen_type(elem), self.gen_expr_const(len))
            }
            Type::Slice(inner) => format!("[{}]", self.gen_type(inner)),
        }
    }

    fn gen_expr_const(&self, e: &Expr) -> String {
        match e {
            Expr::Int(n) => n.to_string(),
            Expr::Ident(s) => s.clone(),
            _ => "0".to_string(),
        }
    }

    // ---- statements & blocks --------------------------------------------------

    fn gen_block(
        &mut self,
        stmts: &[Stmt],
        as_value: bool,
        tail_expected: Option<&Type>,
    ) -> String {
        let n = stmts.len();
        let mut parts: Vec<String> = Vec::new();
        for (i, s) in stmts.iter().enumerate() {
            let is_tail = as_value && i + 1 == n;
            let code = self.gen_stmt(s, is_tail, if is_tail { tail_expected } else { None });
            if !code.is_empty() {
                parts.push(code);
            }
        }
        if parts.is_empty() {
            return "{}".to_string();
        }
        format!("{{\n{}\n}}", indent(&parts.join("\n"), 1))
    }

    fn gen_stmt(&mut self, stmt: &Stmt, is_tail: bool, tail_expected: Option<&Type>) -> String {
        match stmt {
            Stmt::Let {
                mutable,
                name,
                ty,
                value,
            } => {
                let val = self.gen_expr_coerced(value, ty.as_ref());
                if *mutable || ty.is_some() {
                    self.declare(name);
                    let m = if *mutable { "mut " } else { "" };
                    match ty {
                        Some(t) => format!("let {}{}: {} = {};", m, name, self.gen_type(t), val),
                        None => format!("let {}{} = {};", m, name, val),
                    }
                } else if self.declared(name) {
                    format!("{} = {};", name, val)
                } else {
                    self.declare(name);
                    format!("let {} = {};", name, val)
                }
            }
            Stmt::LetTuple { names, value } => {
                for n in names {
                    self.declare(n);
                }
                format!("let ({}) = {};", names.join(", "), self.gen_expr(value))
            }
            Stmt::Assign { target, op, value } => {
                let ops = match op {
                    AssignOp::Eq => "=",
                    AssignOp::Add => "+=",
                    AssignOp::Sub => "-=",
                    AssignOp::Mul => "*=",
                    AssignOp::Div => "/=",
                    AssignOp::Rem => "%=",
                };
                format!(
                    "{} {} {};",
                    self.gen_expr(target),
                    ops,
                    self.gen_expr(value)
                )
            }
            Stmt::Expr(e) => {
                if is_tail {
                    self.gen_expr_coerced(e, tail_expected)
                } else {
                    format!("{};", self.gen_expr(e))
                }
            }
            Stmt::Return(None) => "return;".to_string(),
            Stmt::Return(Some(e)) => {
                let expected = self.cur_ret.clone();
                format!("return {};", self.gen_expr_coerced(e, expected.as_ref()))
            }
            Stmt::If(chain) => self.gen_if(
                &chain.arms,
                chain.else_body.as_deref(),
                is_tail,
                tail_expected,
            ),
            Stmt::While { cond, body } => {
                let b = self.gen_block(body, false, None);
                format!("while {} {}", self.gen_expr(cond), b)
            }
            Stmt::WhileCase {
                pattern,
                expr,
                body,
            } => {
                self.declare_pattern(pattern);
                let b = self.gen_block(body, false, None);
                format!(
                    "while let {} = {} {}",
                    self.gen_pattern(pattern),
                    self.gen_expr(expr),
                    b
                )
            }
            Stmt::For {
                pattern,
                iter,
                iter_borrow,
                body,
            } => {
                self.declare_pattern(pattern);
                let iter_code = match iter_borrow {
                    Borrow::Owned => self.gen_expr(iter),
                    Borrow::Ref => format!("&{}", self.gen_atom(iter)),
                    Borrow::MutRef => format!("&mut {}", self.gen_atom(iter)),
                };
                let b = self.gen_block(body, false, None);
                format!("for {} in {} {}", self.gen_pattern(pattern), iter_code, b)
            }
            Stmt::Loop { label, body } => {
                let b = self.gen_block(body, false, None);
                match label {
                    Some(l) => format!("'{}: loop {}", l, b),
                    None => format!("loop {}", b),
                }
            }
            Stmt::Break { label, value } => {
                let l = label
                    .as_ref()
                    .map(|l| format!(" '{}", l))
                    .unwrap_or_default();
                let v = value
                    .as_ref()
                    .map(|v| format!(" {}", self.gen_expr(v)))
                    .unwrap_or_default();
                format!("break{}{};", l, v)
            }
            Stmt::Continue => "continue;".to_string(),
            Stmt::Match(m) => self.gen_match(m, is_tail, tail_expected),
            Stmt::Unsafe(body) => {
                let b = self.gen_block(body, is_tail, tail_expected);
                format!("unsafe {}", b)
            }
            Stmt::Pass => String::new(),
        }
    }

    fn gen_if(
        &mut self,
        arms: &[(CondKind, Vec<Stmt>)],
        else_body: Option<&[Stmt]>,
        as_value: bool,
        tail_expected: Option<&Type>,
    ) -> String {
        let mut out = String::new();
        for (i, (cond, body)) in arms.iter().enumerate() {
            let head = match cond {
                CondKind::Bool(e) => format!("if {}", self.gen_expr(e)),
                CondKind::Case { pattern, expr } => {
                    self.declare_pattern(pattern);
                    format!(
                        "if let {} = {}",
                        self.gen_pattern(pattern),
                        self.gen_expr(expr)
                    )
                }
            };
            if i == 0 {
                out.push_str(&head);
            } else {
                out.push_str(" else ");
                out.push_str(&head);
            }
            out.push(' ');
            out.push_str(&self.gen_block(body, as_value, tail_expected));
        }
        if let Some(eb) = else_body {
            out.push_str(" else ");
            out.push_str(&self.gen_block(eb, as_value, tail_expected));
        }
        out
    }

    fn gen_match(&mut self, m: &MatchExpr, as_value: bool, tail_expected: Option<&Type>) -> String {
        let scrut = self.gen_expr(&m.scrutinee);
        let mut arms: Vec<String> = Vec::new();
        for arm in &m.arms {
            self.declare_pattern(&arm.pattern);
            let pat = self.gen_pattern(&arm.pattern);
            let guard = arm
                .guard
                .as_ref()
                .map(|g| format!(" if {}", self.gen_expr(g)))
                .unwrap_or_default();
            let body = self.gen_block(&arm.body, as_value, tail_expected);
            arms.push(format!("{}{} => {},", pat, guard, body));
        }
        format!("match {} {{\n{}\n}}", scrut, indent(&arms.join("\n"), 1))
    }

    // ---- patterns -------------------------------------------------------------

    fn declare_pattern(&mut self, pat: &Pattern) {
        match pat {
            Pattern::Ident(n) if n != "_" && n != "none" => self.declare(n),
            Pattern::Tuple(ps) => ps.iter().for_each(|p| self.declare_pattern(p)),
            Pattern::TupleStruct { elems, .. } => {
                elems.iter().for_each(|p| self.declare_pattern(p))
            }
            Pattern::Or(ps) => ps.iter().for_each(|p| self.declare_pattern(p)),
            Pattern::Binding { pattern, name } => {
                self.declare(name);
                self.declare_pattern(pattern);
            }
            _ => {}
        }
    }

    fn gen_pattern(&self, pat: &Pattern) -> String {
        match pat {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Ident(n) if n == "_" => "_".to_string(),
            Pattern::Ident(n) if n == "none" => "None".to_string(),
            Pattern::Ident(n) => n.clone(),
            Pattern::Literal(e) => self.gen_pattern_literal(e),
            Pattern::Tuple(ps) => {
                let inner: Vec<String> = ps.iter().map(|p| self.gen_pattern(p)).collect();
                if inner.len() == 1 {
                    format!("({},)", inner[0])
                } else {
                    format!("({})", inner.join(", "))
                }
            }
            Pattern::TupleStruct { path, elems } => {
                let head = self.gen_pattern_path(path);
                let inner: Vec<String> = elems.iter().map(|p| self.gen_pattern(p)).collect();
                format!("{}({})", head, inner.join(", "))
            }
            Pattern::Path(path) => self.gen_pattern_path(path),
            Pattern::Or(ps) => ps
                .iter()
                .map(|p| self.gen_pattern(p))
                .collect::<Vec<_>>()
                .join(" | "),
            Pattern::Binding { pattern, name } => {
                format!("{} @ {}", name, self.gen_pattern(pattern))
            }
        }
    }

    fn gen_pattern_path(&self, path: &[String]) -> String {
        if path.len() == 1 {
            if path[0] == "none" {
                return "None".to_string();
            }
            return path[0].clone();
        }
        path.join("::")
    }

    fn gen_pattern_literal(&self, e: &Expr) -> String {
        match e {
            Expr::Int(n) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } => format!("-{}", self.gen_pattern_literal(expr)),
            other => self.gen_expr_pure(other),
        }
    }

    // ---- expressions ----------------------------------------------------------

    /// Coerce string literals / collection literals / `if`/`match` tails to an
    /// expected owned type where TinyS implies a conversion Rust wouldn't infer.
    fn gen_expr_coerced(&mut self, e: &Expr, expected: Option<&Type>) -> String {
        let owned_string = expected.map(is_owned_string).unwrap_or(false);
        if owned_string {
            match e {
                Expr::Str(_) | Expr::RawStr(_) => {
                    return format!("{}.to_string()", self.gen_expr(e))
                }
                Expr::If(ie) => {
                    return self.gen_if(&ie.arms, Some(&ie.else_body), true, expected);
                }
                Expr::Match(m) => {
                    return self.gen_match(m, true, expected);
                }
                _ => {}
            }
        }
        if let (Expr::Dict(pairs), Some((kt, vt))) = (e, expected.and_then(dict_kv)) {
            let entries: Vec<String> = pairs
                .iter()
                .map(|(k, v)| {
                    format!(
                        "({}, {})",
                        self.gen_expr_coerced(k, Some(kt)),
                        self.gen_expr_coerced(v, Some(vt))
                    )
                })
                .collect();
            return format!("HashMap::from([{}])", entries.join(", "));
        }
        self.gen_expr(e)
    }

    /// Expression generation that never touches scope (used for constants/patterns).
    fn gen_expr_pure(&self, e: &Expr) -> String {
        match e {
            Expr::Int(n) => n.to_string(),
            Expr::Float(f) => fmt_float(*f),
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::RawStr(s) => s.clone(),
            Expr::Bool(b) => b.to_string(),
            Expr::Ident(n) => n.clone(),
            Expr::NoneLit => "None".to_string(),
            _ => "()".to_string(),
        }
    }

    fn gen_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Int(n) => n.to_string(),
            Expr::Float(f) => fmt_float(*f),
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::RawStr(s) => s.clone(),
            Expr::Bool(b) => b.to_string(),
            Expr::NoneLit => "None".to_string(),
            Expr::Ident(name) => {
                if let Some(real) = self.module_alias.get(name) {
                    real.clone()
                } else {
                    name.clone()
                }
            }
            Expr::Unary { op, expr } => {
                let opstr = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                };
                format!("{}{}", opstr, self.gen_atom(expr))
            }
            Expr::Binary { op, lhs, rhs } => self.gen_binary(*op, lhs, rhs),
            Expr::Field { .. } => self.gen_place(e).0,
            Expr::Index { recv, index } => {
                format!("{}[{}]", self.gen_atom(recv), self.gen_expr(index))
            }
            Expr::Call {
                callee,
                type_args,
                args,
            } => self.gen_call(callee, type_args, args),
            Expr::Ref { mutable, expr } => {
                let m = if *mutable { "mut " } else { "" };
                format!("&{}{}", m, self.gen_atom(expr))
            }
            Expr::Deref(inner) => format!("*{}", self.gen_atom(inner)),
            Expr::CloneExpr(inner) => format!("{}.clone()", self.gen_atom(inner)),
            Expr::Move(inner) => self.gen_expr(inner),
            Expr::Await(inner) => format!("{}.await", self.gen_atom(inner)),
            Expr::Try(inner) => format!("{}?", self.gen_atom(inner)),
            Expr::Range {
                start,
                end,
                inclusive,
            } => {
                let s = start.as_ref().map(|e| self.gen_expr(e)).unwrap_or_default();
                let mid = if *inclusive { "..=" } else { ".." };
                let en = end.as_ref().map(|e| self.gen_expr(e)).unwrap_or_default();
                format!("{}{}{}", s, mid, en)
            }
            Expr::Tuple(elems) => {
                if elems.is_empty() {
                    "()".to_string()
                } else if elems.len() == 1 {
                    format!("({},)", self.gen_expr(&elems[0]))
                } else {
                    let e: Vec<String> = elems.iter().map(|x| self.gen_expr(x)).collect();
                    format!("({})", e.join(", "))
                }
            }
            Expr::List(elems) => {
                let e: Vec<String> = elems.iter().map(|x| self.gen_expr(x)).collect();
                format!("vec![{}]", e.join(", "))
            }
            Expr::Dict(pairs) => {
                let entries: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("({}, {})", self.gen_expr(k), self.gen_expr(v)))
                    .collect();
                format!("HashMap::from([{}])", entries.join(", "))
            }
            Expr::If(ie) => self.gen_if(&ie.arms, Some(&ie.else_body), true, None),
            Expr::Match(m) => self.gen_match(m, true, None),
            Expr::Loop { label, body } => {
                let b = self.gen_block(body, false, None);
                match label {
                    Some(l) => format!("'{}: loop {}", l, b),
                    None => format!("loop {}", b),
                }
            }
            Expr::Closure {
                is_move,
                params,
                ret,
                body,
            } => {
                self.push_scope();
                let ps: Vec<String> = params
                    .iter()
                    .map(|p| {
                        self.declare(&p.name);
                        match &p.ty {
                            Type::Path { segments, args }
                                if args.is_empty() && segments == &["_".to_string()] =>
                            {
                                p.name.clone()
                            }
                            other => format!("{}: {}", p.name, self.gen_type(other)),
                        }
                    })
                    .collect();
                let mv = if *is_move { "move " } else { "" };
                let rt = ret
                    .as_ref()
                    .map(|t| format!(" -> {}", self.gen_type(t)))
                    .unwrap_or_default();
                let block = self.gen_block(body, true, ret.as_ref());
                self.pop_scope();
                format!("{}|{}|{} {}", mv, ps.join(", "), rt, block)
            }
        }
    }

    fn gen_binary(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr) -> String {
        let p = binop_prec(op);
        let l = self.gen_operand(lhs, p, true);
        let r = self.gen_operand(rhs, p, false);
        format!("{} {} {}", l, op.rust(), r)
    }

    fn gen_operand(&mut self, e: &Expr, parent: u8, is_left: bool) -> String {
        match e {
            Expr::Binary { op, .. } => {
                let cp = binop_prec(*op);
                let need = cp < parent || (cp == parent && !is_left);
                let s = self.gen_expr(e);
                if need {
                    format!("({})", s)
                } else {
                    s
                }
            }
            Expr::Range { .. }
            | Expr::If(_)
            | Expr::Match(_)
            | Expr::Loop { .. }
            | Expr::Closure { .. } => {
                format!("({})", self.gen_expr(e))
            }
            _ => self.gen_expr(e),
        }
    }

    /// Wrap an expression in parentheses if it binds looser than a postfix/unary
    /// operator would require.
    fn gen_atom(&mut self, e: &Expr) -> String {
        let low = matches!(
            e,
            Expr::Binary { .. }
                | Expr::Range { .. }
                | Expr::If(_)
                | Expr::Match(_)
                | Expr::Loop { .. }
                | Expr::Closure { .. }
        );
        let s = self.gen_expr(e);
        if low {
            format!("({})", s)
        } else {
            s
        }
    }

    /// Generate a member-access chain, tracking whether the result is a type or
    /// module *path* (needing `::`) versus a value place (needing `.`).
    fn gen_place(&mut self, e: &Expr) -> (String, bool) {
        match e {
            Expr::Ident(name) => {
                if let Some(real) = self.module_alias.get(name) {
                    (real.clone(), true)
                } else {
                    let is_path = starts_upper(name) || self.modules.contains(name);
                    (name.clone(), is_path)
                }
            }
            Expr::Field { recv, name } => {
                let (rc, rp) = self.gen_place(recv);
                let member_is_type = starts_upper(name);
                let use_colon = rp || member_is_type;
                let sep = if use_colon { "::" } else { "." };
                (format!("{}{}{}", rc, sep, name), use_colon)
            }
            _ => (self.gen_atom(e), false),
        }
    }

    fn gen_call(&mut self, callee: &Expr, type_args: &[Type], args: &[Arg]) -> String {
        // Macro invocation?
        if let Expr::Ident(name) = callee {
            if let Some(rust_macro) = self.macro_names.get(name).cloned() {
                return self.gen_macro(&rust_macro, args);
            }
            // `Ok()` with no args → the unit-carrying `Ok(())`.
            if (name == "Ok" || name == "Some") && args.is_empty() {
                return format!("{}(())", name);
            }
        }

        // Struct literal (any keyword argument present)?
        if args.iter().any(|a| matches!(a, Arg::Keyword(_, _))) {
            return self.gen_struct_lit(callee, args);
        }

        let (callee_code, _) = self.gen_place(callee);
        let turbo = if type_args.is_empty() {
            String::new()
        } else {
            let t: Vec<String> = type_args.iter().map(|t| self.gen_type(t)).collect();
            format!("::<{}>", t.join(", "))
        };

        // Coerce positional arguments to known parameter types (owned strings).
        let sig_params: Option<Vec<Type>> = match callee {
            Expr::Ident(n) => self.fn_sigs.get(n).map(|s| s.params.clone()),
            _ => None,
        };
        let arg_strs: Vec<String> = args
            .iter()
            .enumerate()
            .map(|(i, a)| match a {
                Arg::Positional(e) => {
                    let expected = sig_params.as_ref().and_then(|p| p.get(i));
                    self.gen_expr_coerced(e, expected)
                }
                Arg::Keyword(_, e) => self.gen_expr(e),
            })
            .collect();

        format!("{}{}({})", callee_code, turbo, arg_strs.join(", "))
    }

    fn gen_struct_lit(&mut self, callee: &Expr, args: &[Arg]) -> String {
        let (type_code, _) = self.gen_place(callee);
        let simple = type_code
            .rsplit("::")
            .next()
            .unwrap_or(&type_code)
            .to_string();
        let fields = self.struct_fields.get(&simple).cloned();
        let parts: Vec<String> = args
            .iter()
            .filter_map(|a| match a {
                Arg::Keyword(name, value) => {
                    let expected = fields
                        .as_ref()
                        .and_then(|fs| fs.iter().find(|f| &f.name == name))
                        .map(|f| f.ty.clone());
                    Some(format!(
                        "{}: {}",
                        name,
                        self.gen_expr_coerced(value, expected.as_ref())
                    ))
                }
                Arg::Positional(_) => None,
            })
            .collect();
        format!("{} {{ {} }}", type_code, parts.join(", "))
    }

    fn gen_macro(&mut self, rust_macro: &str, args: &[Arg]) -> String {
        let exprs: Vec<&Expr> = args
            .iter()
            .filter_map(|a| match a {
                Arg::Positional(e) => Some(e),
                Arg::Keyword(_, e) => Some(e),
            })
            .collect();

        if rust_macro == "vec" {
            let e: Vec<String> = exprs.iter().map(|x| self.gen_expr(x)).collect();
            return format!("vec![{}]", e.join(", "));
        }

        if matches!(rust_macro, "println" | "print" | "eprintln" | "format") {
            return self.gen_print_like(rust_macro, &exprs);
        }

        let e: Vec<String> = exprs.iter().map(|x| self.gen_expr(x)).collect();
        format!("{}!({})", rust_macro, e.join(", "))
    }

    fn gen_print_like(&mut self, rust_macro: &str, exprs: &[&Expr]) -> String {
        if exprs.is_empty() {
            return format!("{}!()", rust_macro);
        }
        // A leading string literal is treated as the format string.
        if matches!(exprs[0], Expr::Str(_) | Expr::RawStr(_)) {
            let fmt = self.gen_expr(exprs[0]);
            if exprs.len() == 1 {
                return format!("{}!({})", rust_macro, fmt);
            }
            let rest: Vec<String> = exprs[1..].iter().map(|x| self.gen_expr(x)).collect();
            return format!("{}!({}, {})", rust_macro, fmt, rest.join(", "));
        }
        let placeholders = vec!["{}"; exprs.len()].join(" ");
        let vals: Vec<String> = exprs.iter().map(|x| self.gen_expr(x)).collect();
        format!("{}!(\"{}\", {})", rust_macro, placeholders, vals.join(", "))
    }
}

// ---- free helpers -------------------------------------------------------------

fn indent(text: &str, level: usize) -> String {
    let pad = "    ".repeat(level);
    text.lines()
        .map(|l| {
            if l.is_empty() {
                String::new()
            } else {
                format!("{}{}", pad, l)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn starts_upper(s: &str) -> bool {
    s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}

fn map_type_name(name: &str) -> String {
    match name {
        "str" => "String",
        "void" => "()",
        "list" => "Vec",
        "dict" => "HashMap",
        "set" => "HashSet",
        other => other,
    }
    .to_string()
}

fn is_owned_string(ty: &Type) -> bool {
    matches!(ty, Type::Path { segments, args } if args.is_empty()
        && (segments == &["str".to_string()] || segments == &["String".to_string()]))
}

fn dict_kv(ty: &Type) -> Option<(&Type, &Type)> {
    match ty {
        Type::Path { segments, args }
            if args.len() == 2
                && (segments == &["dict".to_string()] || segments == &["HashMap".to_string()]) =>
        {
            Some((&args[0], &args[1]))
        }
        _ => None,
    }
}

fn binop_prec(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Or => 1,
        BinaryOp::And => 2,
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
            3
        }
        BinaryOp::Add | BinaryOp::Sub => 4,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => 5,
    }
}

fn fmt_float(f: f64) -> String {
    let s = format!("{:?}", f);
    s
}
