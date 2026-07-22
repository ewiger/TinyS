//! Recursive-descent parser for TinyS. Consumes the layout-aware token stream
//! from the lexer and produces an [`ast::Program`].

use crate::ast::*;
use crate::diagnostics::{Span, Stage, TinysError};
use crate::lexer::{Tok, Token};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    file: &'a str,
    /// Loop labels currently in scope, used to disambiguate `break <label>` from
    /// `break <value>`.
    labels: Vec<String>,
}

type PResult<T> = Result<T, TinysError>;

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, file: &'a str) -> Self {
        Parser {
            tokens,
            pos: 0,
            file,
            labels: Vec::new(),
        }
    }

    // ---- token cursor helpers -------------------------------------------------

    fn peek(&self) -> &Tok {
        &self.tokens[self.pos].kind
    }

    fn peek2(&self) -> &Tok {
        self.tokens
            .get(self.pos + 1)
            .map(|t| &t.kind)
            .unwrap_or(&Tok::Eof)
    }

    fn span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn is(&self, want: &Tok) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(want)
    }

    fn is2(&self, want: &Tok) -> bool {
        std::mem::discriminant(self.peek2()) == std::mem::discriminant(want)
    }

    fn bump(&mut self) -> Tok {
        let k = self.tokens[self.pos].kind.clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        k
    }

    fn eat(&mut self, want: &Tok) -> bool {
        if self.is(want) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, want: &Tok) -> PResult<()> {
        if self.is(want) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(format!("expected {:?}, found {:?}", want, self.peek())))
        }
    }

    fn expect_ident(&mut self) -> PResult<String> {
        match self.peek().clone() {
            Tok::Ident(s) => {
                self.bump();
                Ok(s)
            }
            other => Err(self.error(format!("expected identifier, found {:?}", other))),
        }
    }

    fn error(&self, msg: impl Into<String>) -> TinysError {
        TinysError::new(Stage::Parse, msg, self.file, self.span())
    }

    fn skip_newlines(&mut self) {
        while self.is(&Tok::Newline) {
            self.bump();
        }
    }

    // ---- program & items ------------------------------------------------------

    pub fn parse_program(&mut self) -> PResult<Program> {
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.is(&Tok::Eof) {
            items.push(self.parse_item()?);
            self.skip_newlines();
        }
        Ok(Program { items })
    }

    fn parse_attrs(&mut self) -> Vec<Attr> {
        let mut attrs = Vec::new();
        loop {
            if let Tok::Attribute(body) = self.peek().clone() {
                attrs.push(body);
                self.bump();
                self.skip_newlines();
            } else {
                break;
            }
        }
        attrs
    }

    fn parse_item(&mut self) -> PResult<Item> {
        let attrs = self.parse_attrs();

        // Imports don't take attributes/pub.
        if self.is(&Tok::From) || self.is(&Tok::Import) {
            return Ok(Item::Use(self.parse_use()?));
        }

        let is_pub = self.eat(&Tok::Pub);
        // `pub[crate]` restricted visibility → we drop the restriction detail for 0.1.0.
        if is_pub && self.is(&Tok::LBracket) {
            self.bump();
            while !self.is(&Tok::RBracket) && !self.is(&Tok::Eof) {
                self.bump();
            }
            self.expect(&Tok::RBracket)?;
        }

        let is_async = self.eat(&Tok::Async);
        let is_unsafe = self.eat(&Tok::Unsafe);

        match self.peek() {
            Tok::Def => {
                let f = self.parse_function(attrs, is_pub, is_async, is_unsafe)?;
                Ok(Item::Function(f))
            }
            Tok::Struct => self.parse_struct(attrs, is_pub),
            Tok::Enum => self.parse_enum(attrs, is_pub),
            Tok::Impl => self.parse_impl(),
            Tok::Trait => self.parse_trait(attrs, is_pub),
            other => Err(self.error(format!("expected item, found {:?}", other))),
        }
    }

    fn parse_use(&mut self) -> PResult<Use> {
        let is_from = self.eat(&Tok::From);
        if !is_from {
            self.expect(&Tok::Import)?;
        }
        // Parse the dotted path.
        let mut path = vec![self.parse_path_seg()?];
        while self.eat(&Tok::Dot) {
            path.push(self.parse_path_seg()?);
        }

        let mut names = Vec::new();
        let mut alias = None;

        if is_from {
            self.expect(&Tok::Import)?;
            loop {
                let name = self.parse_path_seg()?;
                let a = if self.eat(&Tok::As) {
                    Some(self.parse_path_seg()?)
                } else {
                    None
                };
                names.push(UseName { name, alias: a });
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        } else if self.eat(&Tok::As) {
            alias = Some(self.parse_path_seg()?);
        }

        let is_macro = path.first().map(|s| s == "macro").unwrap_or(false);
        let is_rust = path.first().map(|s| s == "rust").unwrap_or(false);
        // Strip the routing root.
        if is_rust || is_macro {
            path.remove(0);
        }

        self.eat(&Tok::Newline);
        Ok(Use {
            path,
            names,
            alias,
            is_macro,
            is_rust,
        })
    }

    /// A path segment may be an identifier or one of the keywords that can also
    /// name a module/macro (e.g. `macro`).
    fn parse_path_seg(&mut self) -> PResult<String> {
        match self.peek().clone() {
            Tok::Ident(s) => {
                self.bump();
                Ok(s)
            }
            _ => Err(self.error("expected a name")),
        }
    }

    fn parse_function(
        &mut self,
        attrs: Vec<Attr>,
        is_pub: bool,
        is_async: bool,
        is_unsafe: bool,
    ) -> PResult<Function> {
        self.expect(&Tok::Def)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generics()?;

        self.expect(&Tok::LParen)?;
        let mut params = Vec::new();
        while !self.is(&Tok::RParen) {
            let pname = match self.peek().clone() {
                Tok::Ident(s) => {
                    self.bump();
                    s
                }
                _ => return Err(self.error("expected parameter name")),
            };
            self.expect(&Tok::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param { name: pname, ty });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen)?;

        let ret = if self.eat(&Tok::Arrow) {
            self.parse_type()?
        } else {
            Type::Unit
        };

        let body = if self.is(&Tok::Colon) {
            Some(self.parse_block()?)
        } else {
            self.eat(&Tok::Newline);
            None
        };

        Ok(Function {
            attrs,
            is_pub,
            is_async,
            is_unsafe,
            name,
            generics,
            params,
            ret,
            body,
        })
    }

    fn parse_struct(&mut self, attrs: Vec<Attr>, is_pub: bool) -> PResult<Item> {
        self.expect(&Tok::Struct)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Tok::Colon)?;
        self.expect(&Tok::Newline)?;
        self.expect(&Tok::Indent)?;

        let mut fields = Vec::new();
        while !self.is(&Tok::Dedent) && !self.is(&Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            if self.eat(&Tok::Pass) {
                self.eat(&Tok::Newline);
                continue;
            }
            let fpub = self.eat(&Tok::Pub);
            let fname = self.expect_ident()?;
            self.expect(&Tok::Colon)?;
            let ty = self.parse_type()?;
            fields.push(Field {
                is_pub: fpub,
                name: fname,
                ty,
            });
            self.eat(&Tok::Newline);
        }
        self.expect(&Tok::Dedent)?;

        Ok(Item::Struct(Struct {
            attrs,
            is_pub,
            name,
            generics,
            fields,
        }))
    }

    fn parse_enum(&mut self, attrs: Vec<Attr>, is_pub: bool) -> PResult<Item> {
        self.expect(&Tok::Enum)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Tok::Colon)?;
        self.expect(&Tok::Newline)?;
        self.expect(&Tok::Indent)?;

        let mut variants = Vec::new();
        while !self.is(&Tok::Dedent) && !self.is(&Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            if self.eat(&Tok::Pass) {
                self.eat(&Tok::Newline);
                continue;
            }
            let vname = self.expect_ident()?;
            let mut payload = Vec::new();
            if self.eat(&Tok::LParen) {
                while !self.is(&Tok::RParen) {
                    payload.push(self.parse_type()?);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RParen)?;
            }
            variants.push(Variant {
                name: vname,
                payload,
            });
            self.eat(&Tok::Newline);
        }
        self.expect(&Tok::Dedent)?;

        Ok(Item::Enum(Enum {
            attrs,
            is_pub,
            name,
            generics,
            variants,
        }))
    }

    fn parse_impl(&mut self) -> PResult<Item> {
        self.expect(&Tok::Impl)?;
        let generics = self.parse_generics()?;
        let first = self.parse_type()?;
        let (trait_name, ty) = if self.eat(&Tok::For) {
            (Some(first), self.parse_type()?)
        } else {
            (None, first)
        };
        self.expect(&Tok::Colon)?;
        self.expect(&Tok::Newline)?;
        self.expect(&Tok::Indent)?;

        let mut methods = Vec::new();
        while !self.is(&Tok::Dedent) && !self.is(&Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            let attrs = self.parse_attrs();
            let mpub = self.eat(&Tok::Pub);
            let masync = self.eat(&Tok::Async);
            let munsafe = self.eat(&Tok::Unsafe);
            methods.push(self.parse_function(attrs, mpub, masync, munsafe)?);
        }
        self.expect(&Tok::Dedent)?;

        Ok(Item::Impl(Impl {
            generics,
            trait_name,
            ty,
            methods,
        }))
    }

    fn parse_trait(&mut self, attrs: Vec<Attr>, is_pub: bool) -> PResult<Item> {
        self.expect(&Tok::Trait)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generics()?;
        self.expect(&Tok::Colon)?;
        self.expect(&Tok::Newline)?;
        self.expect(&Tok::Indent)?;

        let mut methods = Vec::new();
        while !self.is(&Tok::Dedent) && !self.is(&Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            if self.eat(&Tok::Pass) {
                self.eat(&Tok::Newline);
                continue;
            }
            let mattrs = self.parse_attrs();
            let masync = self.eat(&Tok::Async);
            methods.push(self.parse_function(mattrs, false, masync, false)?);
        }
        self.expect(&Tok::Dedent)?;

        Ok(Item::Trait(Trait {
            attrs,
            is_pub,
            name,
            generics,
            methods,
        }))
    }

    // ---- generics & types -----------------------------------------------------

    fn parse_generics(&mut self) -> PResult<Vec<GenericParam>> {
        let mut params = Vec::new();
        if !self.is(&Tok::LBracket) {
            return Ok(params);
        }
        self.bump(); // [
        while !self.is(&Tok::RBracket) {
            match self.peek().clone() {
                Tok::Lifetime(name) => {
                    self.bump();
                    params.push(GenericParam::Lifetime(name));
                }
                Tok::Ident(name) => {
                    self.bump();
                    let mut bounds = Vec::new();
                    if self.eat(&Tok::Colon) {
                        bounds.push(self.parse_bound()?);
                        while self.eat(&Tok::Plus) {
                            bounds.push(self.parse_bound()?);
                        }
                    }
                    params.push(GenericParam::Type { name, bounds });
                }
                other => return Err(self.error(format!("expected generic parameter, found {:?}", other))),
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RBracket)?;
        Ok(params)
    }

    /// A trait bound, e.g. `Clone` or `iter.Iterator` (dotted → `::`).
    fn parse_bound(&mut self) -> PResult<String> {
        let mut segs = vec![self.expect_ident()?];
        while self.eat(&Tok::Dot) {
            segs.push(self.expect_ident()?);
        }
        Ok(segs.join("::"))
    }

    fn parse_type(&mut self) -> PResult<Type> {
        // References: `ref T` / `mut ref T`, with optional `[.lifetime]`.
        if self.is(&Tok::Mut) && self.is2(&Tok::Ref) {
            self.bump();
            self.bump();
            let lifetime = self.parse_opt_lifetime_bracket();
            let inner = Box::new(self.parse_type()?);
            return Ok(Type::Ref {
                mutable: true,
                lifetime,
                inner,
            });
        }
        if self.eat(&Tok::Ref) {
            let lifetime = self.parse_opt_lifetime_bracket();
            let inner = Box::new(self.parse_type()?);
            return Ok(Type::Ref {
                mutable: false,
                lifetime,
                inner,
            });
        }
        if self.eat(&Tok::Void) {
            return Ok(Type::Unit);
        }
        // Tuple type `(A, B)` or unit `()`.
        if self.is(&Tok::LParen) {
            self.bump();
            let mut elems = Vec::new();
            while !self.is(&Tok::RParen) {
                elems.push(self.parse_type()?);
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RParen)?;
            if elems.is_empty() {
                return Ok(Type::Unit);
            }
            return Ok(Type::Tuple(elems));
        }

        // Named / generic type path.
        let mut segments = vec![self.expect_ident()?];
        while self.eat(&Tok::Dot) {
            segments.push(self.expect_ident()?);
        }

        if self.is(&Tok::LBracket) {
            let base = segments.last().cloned().unwrap_or_default();
            self.bump(); // [
            // array[T, N] and slice[T] are special.
            if base == "array" {
                let elem = Box::new(self.parse_type()?);
                self.expect(&Tok::Comma)?;
                let len = Box::new(self.parse_expr()?);
                self.expect(&Tok::RBracket)?;
                return Ok(Type::Array { elem, len });
            }
            if base == "slice" {
                let elem = Box::new(self.parse_type()?);
                self.expect(&Tok::RBracket)?;
                return Ok(Type::Slice(elem));
            }
            let mut args = Vec::new();
            while !self.is(&Tok::RBracket) {
                if let Tok::Lifetime(name) = self.peek().clone() {
                    self.bump();
                    args.push(Type::Path {
                        segments: vec![format!("'{}", name)],
                        args: Vec::new(),
                    });
                } else {
                    args.push(self.parse_type()?);
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RBracket)?;
            return Ok(Type::Path { segments, args });
        }

        Ok(Type::Path {
            segments,
            args: Vec::new(),
        })
    }

    fn parse_opt_lifetime_bracket(&mut self) -> Option<String> {
        if self.is(&Tok::LBracket) {
            if let Tok::Lifetime(name) = self.peek2().clone() {
                self.bump(); // [
                self.bump(); // lifetime
                let _ = self.expect(&Tok::RBracket);
                return Some(name);
            }
        }
        None
    }

    // ---- blocks & statements --------------------------------------------------

    fn parse_block(&mut self) -> PResult<Vec<Stmt>> {
        self.expect(&Tok::Colon)?;
        if self.eat(&Tok::Newline) {
            self.expect(&Tok::Indent)?;
            let mut stmts = Vec::new();
            while !self.is(&Tok::Dedent) && !self.is(&Tok::Eof) {
                if self.eat(&Tok::Newline) {
                    continue;
                }
                stmts.push(self.parse_stmt()?);
            }
            self.expect(&Tok::Dedent)?;
            Ok(stmts)
        } else {
            // Inline single statement, e.g. `case _: pass`.
            Ok(vec![self.parse_stmt()?])
        }
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        match self.peek() {
            Tok::Return => {
                self.bump();
                let value = if self.stmt_terminated() {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.eat(&Tok::Newline);
                Ok(Stmt::Return(value))
            }
            Tok::Pass => {
                self.bump();
                self.eat(&Tok::Newline);
                Ok(Stmt::Pass)
            }
            Tok::Break => {
                self.bump();
                let label = match self.peek().clone() {
                    Tok::Ident(s) if self.labels.contains(&s) => {
                        self.bump();
                        Some(s)
                    }
                    _ => None,
                };
                let value = if self.eat(&Tok::With) {
                    Some(self.parse_expr()?)
                } else if self.stmt_terminated() {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.eat(&Tok::Newline);
                Ok(Stmt::Break { label, value })
            }
            Tok::Continue => {
                self.bump();
                self.eat(&Tok::Newline);
                Ok(Stmt::Continue)
            }
            Tok::If => self.parse_if_stmt(),
            Tok::While => self.parse_while(),
            Tok::For => self.parse_for(),
            Tok::Loop => self.parse_loop(),
            Tok::Match => {
                let m = self.parse_match()?;
                Ok(Stmt::Match(m))
            }
            Tok::Unsafe => {
                self.bump();
                let body = self.parse_block()?;
                Ok(Stmt::Unsafe(body))
            }
            Tok::Mut => {
                self.bump();
                let name = self.expect_ident()?;
                let ty = if self.eat(&Tok::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(&Tok::Assign)?;
                let value = self.parse_expr()?;
                self.eat(&Tok::Newline);
                Ok(Stmt::Let {
                    mutable: true,
                    name,
                    ty,
                    value,
                })
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn stmt_terminated(&self) -> bool {
        matches!(self.peek(), Tok::Newline | Tok::Dedent | Tok::Eof)
    }

    fn parse_expr_stmt(&mut self) -> PResult<Stmt> {
        let first = self.parse_expr()?;

        // Tuple destructuring: `a, b = e`.
        if self.is(&Tok::Comma) {
            let mut names = vec![self.expr_as_name(&first)?];
            while self.eat(&Tok::Comma) {
                let e = self.parse_expr()?;
                names.push(self.expr_as_name(&e)?);
            }
            self.expect(&Tok::Assign)?;
            let value = self.parse_expr()?;
            self.eat(&Tok::Newline);
            return Ok(Stmt::LetTuple { names, value });
        }

        // Typed binding: `name: T = e`.
        if self.is(&Tok::Colon) {
            if let Expr::Ident(name) = &first {
                let name = name.clone();
                self.bump(); // :
                let ty = self.parse_type()?;
                self.expect(&Tok::Assign)?;
                let value = self.parse_expr()?;
                self.eat(&Tok::Newline);
                return Ok(Stmt::Let {
                    mutable: false,
                    name,
                    ty: Some(ty),
                    value,
                });
            }
        }

        // Plain `=` binding/assignment.
        if self.eat(&Tok::Assign) {
            let value = self.parse_expr()?;
            self.eat(&Tok::Newline);
            return Ok(match first {
                Expr::Ident(name) => Stmt::Let {
                    mutable: false,
                    name,
                    ty: None,
                    value,
                },
                target => Stmt::Assign {
                    target,
                    op: AssignOp::Eq,
                    value,
                },
            });
        }

        // Compound assignment.
        if let Some(op) = self.compound_op() {
            self.bump();
            let value = self.parse_expr()?;
            self.eat(&Tok::Newline);
            return Ok(Stmt::Assign {
                target: first,
                op,
                value,
            });
        }

        self.eat(&Tok::Newline);
        Ok(Stmt::Expr(first))
    }

    fn expr_as_name(&self, e: &Expr) -> PResult<String> {
        match e {
            Expr::Ident(s) => Ok(s.clone()),
            _ => Err(self.error("expected a name in destructuring assignment")),
        }
    }

    fn compound_op(&self) -> Option<AssignOp> {
        match self.peek() {
            Tok::PlusEq => Some(AssignOp::Add),
            Tok::MinusEq => Some(AssignOp::Sub),
            Tok::StarEq => Some(AssignOp::Mul),
            Tok::SlashEq => Some(AssignOp::Div),
            Tok::PercentEq => Some(AssignOp::Rem),
            _ => None,
        }
    }

    fn parse_if_stmt(&mut self) -> PResult<Stmt> {
        let (arms, else_body) = self.parse_if_common()?;
        Ok(Stmt::If(IfChain { arms, else_body }))
    }

    /// Shared parsing for `if`/`elif`/`else` used by both statement and
    /// expression positions.
    fn parse_if_common(&mut self) -> PResult<(Vec<(CondKind, Vec<Stmt>)>, Option<Vec<Stmt>>)> {
        let mut arms = Vec::new();
        self.expect(&Tok::If)?;
        let cond = self.parse_cond()?;
        let body = self.parse_block()?;
        arms.push((cond, body));

        loop {
            self.skip_newlines_before_keyword(&Tok::Elif);
            if self.eat(&Tok::Elif) {
                let cond = self.parse_cond()?;
                let body = self.parse_block()?;
                arms.push((cond, body));
            } else {
                break;
            }
        }

        self.skip_newlines_before_keyword(&Tok::Else);
        let else_body = if self.eat(&Tok::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok((arms, else_body))
    }

    /// After a block's Dedent, an `elif`/`else` may follow on the next line;
    /// tolerate an intervening Newline.
    fn skip_newlines_before_keyword(&mut self, kw: &Tok) {
        while self.is(&Tok::Newline) && std::mem::discriminant(self.peek2()) == std::mem::discriminant(kw)
        {
            self.bump();
        }
    }

    fn parse_cond(&mut self) -> PResult<CondKind> {
        if self.eat(&Tok::Case) {
            let pattern = self.parse_pattern()?;
            self.expect(&Tok::Assign)?;
            let expr = self.parse_expr()?;
            Ok(CondKind::Case { pattern, expr })
        } else {
            Ok(CondKind::Bool(self.parse_expr()?))
        }
    }

    fn parse_while(&mut self) -> PResult<Stmt> {
        self.expect(&Tok::While)?;
        if self.eat(&Tok::Case) {
            let pattern = self.parse_pattern()?;
            self.expect(&Tok::Assign)?;
            let expr = self.parse_expr()?;
            let body = self.parse_block()?;
            return Ok(Stmt::WhileCase {
                pattern,
                expr,
                body,
            });
        }
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_for(&mut self) -> PResult<Stmt> {
        self.expect(&Tok::For)?;
        let pattern = self.parse_pattern()?;
        self.expect(&Tok::In)?;
        let iter_borrow = if self.is(&Tok::Mut) && self.is2(&Tok::Ref) {
            self.bump();
            self.bump();
            Borrow::MutRef
        } else if self.eat(&Tok::Ref) {
            Borrow::Ref
        } else {
            Borrow::Owned
        };
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For {
            pattern,
            iter,
            iter_borrow,
            body,
        })
    }

    fn parse_loop(&mut self) -> PResult<Stmt> {
        let (label, body) = self.parse_loop_parts()?;
        Ok(Stmt::Loop { label, body })
    }

    fn parse_loop_parts(&mut self) -> PResult<(Option<String>, Vec<Stmt>)> {
        self.expect(&Tok::Loop)?;
        let label = if self.eat(&Tok::As) {
            Some(self.expect_ident()?)
        } else {
            None
        };
        if let Some(l) = &label {
            self.labels.push(l.clone());
        }
        let body = self.parse_block()?;
        if label.is_some() {
            self.labels.pop();
        }
        Ok((label, body))
    }

    // ---- patterns -------------------------------------------------------------

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        let mut pats = vec![self.parse_pattern_atom()?];
        while self.eat(&Tok::Pipe) {
            pats.push(self.parse_pattern_atom()?);
        }
        let mut p = if pats.len() == 1 {
            pats.pop().unwrap()
        } else {
            Pattern::Or(pats)
        };
        if self.eat(&Tok::As) {
            let name = self.expect_ident()?;
            p = Pattern::Binding {
                pattern: Box::new(p),
                name,
            };
        }
        Ok(p)
    }

    fn parse_pattern_atom(&mut self) -> PResult<Pattern> {
        match self.peek().clone() {
            Tok::Int(n) => {
                self.bump();
                Ok(Pattern::Literal(Expr::Int(n)))
            }
            Tok::Minus => {
                self.bump();
                if let Tok::Int(n) = self.peek().clone() {
                    self.bump();
                    Ok(Pattern::Literal(Expr::Int(-n)))
                } else {
                    Err(self.error("expected number after `-` in pattern"))
                }
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Pattern::Literal(Expr::Str(s)))
            }
            Tok::True => {
                self.bump();
                Ok(Pattern::Literal(Expr::Bool(true)))
            }
            Tok::False => {
                self.bump();
                Ok(Pattern::Literal(Expr::Bool(false)))
            }
            Tok::LParen => {
                self.bump();
                let mut elems = Vec::new();
                while !self.is(&Tok::RParen) {
                    elems.push(self.parse_pattern()?);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RParen)?;
                Ok(Pattern::Tuple(elems))
            }
            Tok::Ident(first) => {
                self.bump();
                let mut path = vec![first];
                while self.eat(&Tok::Dot) {
                    path.push(self.expect_ident()?);
                }
                if self.eat(&Tok::LParen) {
                    let mut elems = Vec::new();
                    while !self.is(&Tok::RParen) {
                        elems.push(self.parse_pattern()?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(&Tok::RParen)?;
                    Ok(Pattern::TupleStruct { path, elems })
                } else if path.len() == 1 {
                    Ok(Pattern::Ident(path.pop().unwrap()))
                } else {
                    Ok(Pattern::Path(path))
                }
            }
            other => Err(self.error(format!("expected pattern, found {:?}", other))),
        }
    }

    // ---- expressions ----------------------------------------------------------

    fn parse_expr(&mut self) -> PResult<Expr> {
        match self.peek() {
            Tok::If => {
                let (arms, else_body) = self.parse_if_common()?;
                let else_body = else_body
                    .ok_or_else(|| self.error("`if` used as a value must have an `else` branch"))?;
                Ok(Expr::If(Box::new(IfExpr { arms, else_body })))
            }
            Tok::Match => {
                let m = self.parse_match()?;
                Ok(Expr::Match(Box::new(m)))
            }
            Tok::Fn => self.parse_closure(false),
            Tok::Move if self.is2(&Tok::Fn) => {
                self.bump();
                self.parse_closure(true)
            }
            Tok::Loop => {
                let (label, body) = self.parse_loop_parts()?;
                Ok(Expr::Loop { label, body })
            }
            _ => self.parse_range(),
        }
    }

    fn parse_closure(&mut self, is_move: bool) -> PResult<Expr> {
        self.expect(&Tok::Fn)?;
        self.expect(&Tok::LParen)?;
        let mut params = Vec::new();
        while !self.is(&Tok::RParen) {
            let name = self.expect_ident()?;
            let ty = if self.eat(&Tok::Colon) {
                self.parse_type()?
            } else {
                Type::Path {
                    segments: vec!["_".to_string()],
                    args: Vec::new(),
                }
            };
            params.push(Param { name, ty });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen)?;
        let ret = if self.eat(&Tok::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Expr::Closure {
            is_move,
            params,
            ret,
            body,
        })
    }

    fn parse_range(&mut self) -> PResult<Expr> {
        let start = self.parse_or()?;
        if self.is(&Tok::DotDot) || self.is(&Tok::DotDotEq) {
            let inclusive = self.is(&Tok::DotDotEq);
            self.bump();
            let end = if self.range_end_follows() {
                Some(Box::new(self.parse_or()?))
            } else {
                None
            };
            return Ok(Expr::Range {
                start: Some(Box::new(start)),
                end,
                inclusive,
            });
        }
        Ok(start)
    }

    fn range_end_follows(&self) -> bool {
        !matches!(
            self.peek(),
            Tok::Newline | Tok::Dedent | Tok::Eof | Tok::Colon | Tok::RParen | Tok::RBracket
        )
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_and()?;
        while self.eat(&Tok::Or) {
            let rhs = self.parse_and()?;
            lhs = Expr::Binary {
                op: BinaryOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_not()?;
        while self.eat(&Tok::And) {
            let rhs = self.parse_not()?;
            lhs = Expr::Binary {
                op: BinaryOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> PResult<Expr> {
        if self.eat(&Tok::Not) {
            let expr = self.parse_not()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Tok::EqEq => BinaryOp::Eq,
                Tok::NotEq => BinaryOp::Ne,
                Tok::Lt => BinaryOp::Lt,
                Tok::Gt => BinaryOp::Gt,
                Tok::Le => BinaryOp::Le,
                Tok::Ge => BinaryOp::Ge,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_additive()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_additive(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinaryOp::Add,
                Tok::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_multiplicative()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinaryOp::Mul,
                Tok::Slash => BinaryOp::Div,
                Tok::Percent => BinaryOp::Rem,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_unary()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        match self.peek() {
            Tok::Minus => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            Tok::Ref => {
                self.bump();
                Ok(Expr::Ref {
                    mutable: false,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            Tok::Mut if self.is2(&Tok::Ref) => {
                self.bump();
                self.bump();
                Ok(Expr::Ref {
                    mutable: true,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            Tok::At => {
                self.bump();
                Ok(Expr::Deref(Box::new(self.parse_unary()?)))
            }
            Tok::Clone => {
                self.bump();
                Ok(Expr::CloneExpr(Box::new(self.parse_unary()?)))
            }
            Tok::Move => {
                self.bump();
                Ok(Expr::Move(Box::new(self.parse_unary()?)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let mut e = self.parse_primary()?;
        loop {
            match self.peek() {
                Tok::LParen => {
                    let args = self.parse_call_args()?;
                    e = Expr::Call {
                        callee: Box::new(e),
                        type_args: Vec::new(),
                        args,
                    };
                }
                Tok::LBracket => {
                    if self.is_turbofish() {
                        self.bump(); // [
                        let mut type_args = Vec::new();
                        while !self.is(&Tok::RBracket) {
                            type_args.push(self.parse_type()?);
                            if !self.eat(&Tok::Comma) {
                                break;
                            }
                        }
                        self.expect(&Tok::RBracket)?;
                        let args = self.parse_call_args()?;
                        e = Expr::Call {
                            callee: Box::new(e),
                            type_args,
                            args,
                        };
                    } else {
                        self.bump(); // [
                        let index = self.parse_expr()?;
                        self.expect(&Tok::RBracket)?;
                        e = Expr::Index {
                            recv: Box::new(e),
                            index: Box::new(index),
                        };
                    }
                }
                Tok::Dot => {
                    self.bump();
                    if self.eat(&Tok::Await) {
                        e = Expr::Await(Box::new(e));
                    } else {
                        let name = self.parse_member_name()?;
                        e = Expr::Field {
                            recv: Box::new(e),
                            name,
                        };
                    }
                }
                Tok::Question => {
                    self.bump();
                    e = Expr::Try(Box::new(e));
                }
                _ => break,
            }
        }
        Ok(e)
    }

    /// A member name after `.` is usually an identifier or a tuple index, but a
    /// few keywords (notably `clone`) may name methods.
    fn parse_member_name(&mut self) -> PResult<String> {
        match self.peek().clone() {
            Tok::Ident(s) => {
                self.bump();
                Ok(s)
            }
            Tok::Int(n) => {
                self.bump();
                Ok(n.to_string())
            }
            Tok::Clone => {
                self.bump();
                Ok("clone".to_string())
            }
            other => Err(self.error(format!("expected field or method name, found {:?}", other))),
        }
    }

    /// Distinguish `f[T](...)` (generic args, turbofish) from `xs[i]` (index) by
    /// scanning to the matching `]` and checking whether a `(` follows.
    fn is_turbofish(&self) -> bool {
        let mut depth = 0;
        let mut i = self.pos;
        loop {
            match self.tokens.get(i).map(|t| &t.kind) {
                Some(Tok::LBracket) => depth += 1,
                Some(Tok::RBracket) => {
                    depth -= 1;
                    if depth == 0 {
                        return matches!(
                            self.tokens.get(i + 1).map(|t| &t.kind),
                            Some(Tok::LParen)
                        );
                    }
                }
                Some(Tok::Eof) | None => return false,
                _ => {}
            }
            i += 1;
        }
    }

    fn parse_call_args(&mut self) -> PResult<Vec<Arg>> {
        self.expect(&Tok::LParen)?;
        let mut args = Vec::new();
        while !self.is(&Tok::RParen) {
            // Keyword argument `name=value` → struct-literal field.
            if let Tok::Ident(name) = self.peek().clone() {
                if self.is2(&Tok::Assign) {
                    self.bump(); // name
                    self.bump(); // =
                    let value = self.parse_expr()?;
                    args.push(Arg::Keyword(name, value));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                    continue;
                }
            }
            args.push(Arg::Positional(self.parse_expr()?));
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen)?;
        Ok(args)
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        match self.peek().clone() {
            Tok::Int(n) => {
                self.bump();
                Ok(Expr::Int(n))
            }
            Tok::Float(f) => {
                self.bump();
                Ok(Expr::Float(f))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Expr::Str(s))
            }
            Tok::RawStr(s) => {
                self.bump();
                Ok(Expr::RawStr(s))
            }
            Tok::True => {
                self.bump();
                Ok(Expr::Bool(true))
            }
            Tok::False => {
                self.bump();
                Ok(Expr::Bool(false))
            }
            Tok::Ident(name) => {
                self.bump();
                if name == "none" {
                    Ok(Expr::NoneLit)
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Tok::LParen => {
                self.bump();
                let mut elems = Vec::new();
                let mut trailing_comma = false;
                while !self.is(&Tok::RParen) {
                    elems.push(self.parse_expr()?);
                    if self.eat(&Tok::Comma) {
                        trailing_comma = true;
                    } else {
                        trailing_comma = false;
                        break;
                    }
                }
                self.expect(&Tok::RParen)?;
                if elems.len() == 1 && !trailing_comma {
                    Ok(elems.pop().unwrap())
                } else {
                    Ok(Expr::Tuple(elems))
                }
            }
            Tok::LBracket => {
                self.bump();
                let mut elems = Vec::new();
                while !self.is(&Tok::RBracket) {
                    elems.push(self.parse_expr()?);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RBracket)?;
                Ok(Expr::List(elems))
            }
            Tok::LBrace => {
                self.bump();
                let mut pairs = Vec::new();
                while !self.is(&Tok::RBrace) {
                    let key = self.parse_expr()?;
                    self.expect(&Tok::Colon)?;
                    let value = self.parse_expr()?;
                    pairs.push((key, value));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RBrace)?;
                Ok(Expr::Dict(pairs))
            }
            other => Err(self.error(format!("expected expression, found {:?}", other))),
        }
    }

    fn parse_match(&mut self) -> PResult<MatchExpr> {
        self.expect(&Tok::Match)?;
        let scrutinee = Box::new(self.parse_expr()?);
        self.expect(&Tok::Colon)?;
        self.expect(&Tok::Newline)?;
        self.expect(&Tok::Indent)?;

        let mut arms = Vec::new();
        while !self.is(&Tok::Dedent) && !self.is(&Tok::Eof) {
            if self.eat(&Tok::Newline) {
                continue;
            }
            self.expect(&Tok::Case)?;
            let pattern = self.parse_pattern()?;
            let guard = if self.eat(&Tok::If) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            let body = self.parse_block()?;
            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });
        }
        self.expect(&Tok::Dedent)?;

        Ok(MatchExpr { scrutinee, arms })
    }
}
