//! Codegen tests: assert on the generated Rust text. These are fast and do not
//! invoke `rustc`. See `tests/examples.rs` for full compile-and-run coverage.

use tinys::compile_to_rust;

fn gen(src: &str) -> String {
    compile_to_rust(src, "test.sn").unwrap_or_else(|e| panic!("compile failed: {e}"))
}

fn assert_has(src: &str, needle: &str) {
    let out = gen(src);
    assert!(
        out.contains(needle),
        "expected generated Rust to contain:\n  {needle}\n--- generated ---\n{out}"
    );
}

#[test]
fn hello_world() {
    assert_has(
        "def main() -> void:\n    print(\"Hello from TinyS\")\n",
        "fn main()",
    );
    assert_has(
        "def main() -> void:\n    print(\"Hello from TinyS\")\n",
        "println!(\"Hello from TinyS\")",
    );
}

#[test]
fn function_signature_and_return() {
    assert_has(
        "def add(left: i64, right: i64) -> i64:\n    return left + right\n",
        "fn add(left: i64, right: i64) -> i64",
    );
    assert_has(
        "def add(left: i64, right: i64) -> i64:\n    return left + right\n",
        "return left + right;",
    );
}

#[test]
fn implicit_tail_return_has_no_semicolon() {
    let out = gen("def square(v: i32) -> i32:\n    v * v\n");
    assert!(out.contains("v * v"));
    assert!(!out.contains("v * v;"), "tail expression must not be a statement:\n{out}");
}

#[test]
fn void_function_keeps_trailing_statement() {
    // A trailing expression in a `void` function stays a statement (yields ()).
    assert_has("def main() -> void:\n    print(1)\n", "println!(\"{}\", 1);");
}

#[test]
fn mutability_and_binding() {
    assert_has("def f() -> void:\n    x = 5\n", "let x = 5;");
    assert_has("def f() -> void:\n    mut total = 0\n", "let mut total = 0;");
    assert_has("def f() -> void:\n    count: i32 = 0\n", "let count: i32 = 0;");
}

#[test]
fn reassignment_of_existing_binding_omits_let() {
    let out = gen("def f() -> void:\n    mut total = 0\n    total = 5\n");
    assert!(out.contains("let mut total = 0;"));
    assert!(out.contains("total = 5;"));
    assert!(!out.contains("let total = 5;"), "reassignment must not re-`let`:\n{out}");
}

#[test]
fn compound_assignment() {
    assert_has("def f() -> void:\n    mut n = 0\n    n += 3\n", "n += 3;");
}

#[test]
fn ranges_loops() {
    assert_has("def f() -> void:\n    for i in 0..=5:\n        print(i)\n", "for i in 0..=5");
    assert_has("def f() -> void:\n    for i in 0..10:\n        print(i)\n", "for i in 0..10");
    assert_has("def f() -> void:\n    mut n = 3\n    while n > 0:\n        n -= 1\n", "while n > 0");
    assert_has("def f() -> void:\n    loop:\n        break\n", "loop {");
}

#[test]
fn if_elif_else_statement() {
    let src = "def f(n: i32) -> void:\n    if n > 0:\n        print(1)\n    elif n < 0:\n        print(2)\n    else:\n        print(3)\n";
    assert_has(src, "if n > 0 {");
    assert_has(src, "} else if n < 0 {");
    assert_has(src, "} else {");
}

#[test]
fn if_expression() {
    assert_has(
        "def f(a: bool) -> void:\n    status = if a:\n        \"x\"\n    else:\n        \"y\"\n",
        "let status = if a {",
    );
}

#[test]
fn boolean_and_comparison_operators() {
    assert_has("def f(a: bool, b: bool) -> void:\n    c = a and b\n", "a && b");
    assert_has("def f(a: bool, b: bool) -> void:\n    c = a or b\n", "a || b");
    assert_has("def f(a: bool) -> void:\n    c = not a\n", "!a");
}

#[test]
fn arithmetic_precedence_is_preserved() {
    assert_has("def f() -> void:\n    x = 1 + 2 * 3\n", "1 + 2 * 3");
    assert_has("def f() -> void:\n    x = (1 + 2) * 3\n", "(1 + 2) * 3");
}

#[test]
fn structs_and_construction() {
    let src = "struct Point:\n    x: f64\n    y: f64\n\ndef f() -> void:\n    p = Point(x=1.0, y=2.0)\n";
    assert_has(src, "struct Point {");
    assert_has(src, "x: f64,");
    assert_has(src, "Point { x: 1.0, y: 2.0 }");
}

#[test]
fn owned_string_field_is_coerced() {
    let src = "struct User:\n    name: str\n\ndef f() -> void:\n    u = User(name=\"Ada\")\n";
    assert_has(src, "name: String,");
    assert_has(src, "name: \"Ada\".to_string()");
}

#[test]
fn methods_and_receivers() {
    let src = "struct P:\n    x: f64\n\nimpl P:\n\n    def get(self: ref Self) -> f64:\n        return self.x\n\n    def set(self: mut ref Self, v: f64) -> void:\n        self.x = v\n";
    assert_has(src, "impl P {");
    assert_has(src, "fn get(&self) -> f64");
    assert_has(src, "fn set(&mut self, v: f64)");
    assert_has(src, "self.x");
}

#[test]
fn associated_function_uses_colon_colon() {
    assert_has(
        "def f() -> void:\n    p = Point.new(1.0, 2.0)\n",
        "Point::new(1.0, 2.0)",
    );
}

#[test]
fn enums_and_match() {
    let src = "enum Token:\n    Number(i64)\n    Plus\n    End\n\ndef f(t: Token) -> void:\n    match t:\n        case Token.Number(v):\n            print(v)\n        case Token.Plus:\n            print(1)\n        case Token.End:\n            pass\n";
    assert_has(src, "enum Token {");
    assert_has(src, "Number(i64),");
    assert_has(src, "match t {");
    assert_has(src, "Token::Number(v) =>");
    assert_has(src, "Token::Plus =>");
}

#[test]
fn match_guard_or_and_binding() {
    assert_has(
        "def f(n: i32) -> void:\n    match n:\n        case 0 if n == 0:\n            print(1)\n        case _:\n            pass\n",
        "0 if n == 0 =>",
    );
    assert_has(
        "def f(t: i32) -> void:\n    match t:\n        case 1 | 2:\n            print(1)\n        case _:\n            pass\n",
        "1 | 2 =>",
    );
    assert_has(
        "def f(t: Token) -> void:\n    match t:\n        case Token.N(v) as whole:\n            print(1)\n        case _:\n            pass\n",
        "whole @ Token::N(v) =>",
    );
}

#[test]
fn references_and_deref() {
    assert_has("def f(x: ref i32) -> void:\n    pass\n", "x: &i32");
    assert_has("def f(x: mut ref i32) -> void:\n    pass\n", "x: &mut i32");
    assert_has("def f(x: ref str) -> void:\n    pass\n", "x: &str");
    assert_has("def f(x: mut ref i32) -> void:\n    at x += 1\n", "*x += 1;");
    assert_has("def f() -> void:\n    mut n = 0\n    r = mut ref n\n", "&mut n");
}

#[test]
fn none_and_some() {
    assert_has("def f() -> void:\n    x = none\n", "let x = None;");
    assert_has("def f() -> void:\n    x = Some(5)\n", "Some(5)");
}

#[test]
fn if_case_and_while_case() {
    assert_has(
        "def f() -> void:\n    if case Some(u) = lookup():\n        print(u)\n",
        "if let Some(u) = lookup()",
    );
    assert_has(
        "def f() -> void:\n    while case Some(x) = it.next():\n        print(x)\n",
        "while let Some(x) = it.next()",
    );
}

#[test]
fn generics_and_bounds() {
    assert_has(
        "def identity[T](value: T) -> T:\n    return value\n",
        "fn identity<T>(value: T) -> T",
    );
    assert_has(
        "def clone_it[T: Clone](value: ref T) -> T:\n    return clone value\n",
        "fn clone_it<T: Clone>(value: &T) -> T",
    );
}

#[test]
fn generic_type_applications() {
    assert_has("def f(v: list[i32]) -> void:\n    pass\n", "v: Vec<i32>");
    assert_has("def f(v: dict[str, i32]) -> void:\n    pass\n", "v: HashMap<String, i32>");
    assert_has("def f() -> Option[i32]:\n    return none\n", "-> Option<i32>");
}

#[test]
fn collections_literals() {
    assert_has("def f() -> void:\n    xs = [1, 2, 3]\n", "vec![1, 2, 3]");
    assert_has(
        "def f() -> void:\n    m = {\"one\": 1, \"two\": 2}\n",
        "HashMap::from([(\"one\", 1), (\"two\", 2)])",
    );
    assert_has("def f() -> void:\n    m = {\"one\": 1}\n", "use std::collections::HashMap;");
}

#[test]
fn tuples_and_destructuring() {
    assert_has("def f() -> void:\n    pair = (\"Ada\", 42)\n", "(\"Ada\", 42)");
    assert_has(
        "def f(pair: (str, i32)) -> void:\n    name, age = pair\n",
        "let (name, age) = pair;",
    );
}

#[test]
fn closures() {
    assert_has(
        "def f() -> void:\n    double = fn(x: i32) -> i32:\n        x * 2\n",
        "|x: i32| -> i32 {",
    );
    assert_has(
        "def f(data: i32) -> void:\n    w = move fn():\n        print(data)\n",
        "move ||",
    );
}

#[test]
fn error_propagation_operator() {
    assert_has(
        "def f() -> Result[i32, E]:\n    x = parse(source)?\n    return Ok(x)\n",
        "parse(source)?",
    );
}

#[test]
fn imports_rust_and_alias() {
    assert_has(
        "from rust.std.collections import HashMap\n\ndef f() -> void:\n    pass\n",
        "use std::collections::HashMap;",
    );
    assert_has(
        "from rust.serde import Serialize, Deserialize\n\ndef f() -> void:\n    pass\n",
        "use serde::{Serialize, Deserialize};",
    );
    let src = "import rust.serde_json as json\n\ndef f(s: ref str) -> void:\n    x = json.from_str[User](s)\n";
    assert_has(src, "use serde_json;");
    assert_has(src, "serde_json::from_str::<User>(s)");
}

#[test]
fn macros() {
    assert_has("def f(x: i32) -> void:\n    debug(x)\n", "dbg!(x)");
    assert_has("def f(x: i32) -> void:\n    y = format(\"{}\", x)\n", "format!(\"{}\", x)");
    assert_has("def f() -> void:\n    v = vec(1, 2, 3)\n", "vec![1, 2, 3]");
    assert_has("def f(x: i32) -> void:\n    assert(x > 0)\n", "assert!(x > 0)");
}

#[test]
fn macro_imports_emit_no_use_lines() {
    let src = "from macro import assert, debug, format\nfrom macro.std import vec\n\ndef f(x: i32) -> void:\n    debug(x)\n";
    let out = gen(src);
    assert!(
        !out.contains("use macro"),
        "the `macro` root is routing only, never a Rust path:\n{out}"
    );
    assert!(!out.contains("use std::vec"), "std macros need no import:\n{out}");
}

#[test]
fn macro_imports_from_the_std_root_stay_unqualified() {
    assert_has(
        "from macro.std import vec\n\ndef f() -> void:\n    v = vec(1, 2)\n",
        "vec![1, 2]",
    );
    assert_has(
        "from macro import assert_eq\n\ndef f(x: i32) -> void:\n    assert_eq(x, 1)\n",
        "assert_eq!(x, 1)",
    );
    // `print` is a macro too, so importing it explicitly must not change it into
    // a function call.
    assert_has(
        "from macro import print, eprint\n\ndef f() -> void:\n    print(\"hi\")\n    eprint(\"oops\")\n",
        "println!(\"hi\")",
    );
    assert_has(
        "from macro import print, eprint\n\ndef f() -> void:\n    print(\"hi\")\n    eprint(\"oops\")\n",
        "eprintln!(\"oops\")",
    );
}

#[test]
fn macro_imports_are_aliasable() {
    // The alias becomes the call-site name; the Rust macro behind it is unchanged.
    assert_has(
        "from macro import assert as require\n\ndef f(x: i32) -> void:\n    require(x > 0)\n",
        "assert!(x > 0)",
    );
    assert_has(
        "from macro import debug as inspect\n\ndef f(x: i32) -> void:\n    inspect(x)\n",
        "dbg!(x)",
    );
}

#[test]
fn crate_macro_imports_are_path_qualified() {
    // `macro.<crate>` is a crate namespace, so the call site must name the crate —
    // a bare `json!(...)` would not resolve in the generated Rust.
    assert_has(
        "from macro.serde_json import json\n\ndef f() -> void:\n    v = json(1)\n",
        "serde_json::json!(1)",
    );
    assert_has(
        "from macro.regex import regex\n\ndef f() -> void:\n    r = regex(\"a+\")\n",
        "regex::regex!(\"a+\")",
    );
    assert_has(
        "from macro.serde_json import json as j\n\ndef f() -> void:\n    v = j(1)\n",
        "serde_json::json!(1)",
    );
}

#[test]
fn imported_macro_names_do_not_become_function_calls() {
    let out = gen("from macro import panic\n\ndef f() -> void:\n    panic(\"boom\")\n");
    assert!(out.contains("panic!("), "expected a macro invocation:\n{out}");
}

#[test]
fn attributes_and_visibility() {
    assert_has(
        "#[derive(Debug, Clone)]\nstruct User:\n    id: u64\n",
        "#[derive(Debug, Clone)]",
    );
    assert_has("pub def f() -> void:\n    pass\n", "pub fn f()");
    assert_has("pub struct User:\n    pub id: u64\n", "pub struct User");
}

#[test]
fn async_await() {
    assert_has(
        "async def fetch() -> void:\n    pass\n",
        "async fn fetch()",
    );
    assert_has(
        "async def fetch(c: Client) -> void:\n    r = c.get().await\n",
        "c.get().await",
    );
}

#[test]
fn labeled_loop_and_break_with_value() {
    let src = "def f() -> void:\n    result = loop as search:\n        for item in items:\n            break search with item\n";
    assert_has(src, "'search: loop {");
    assert_has(src, "break 'search item;");
}

#[test]
fn traits() {
    let src = "trait Display:\n    def show(self: ref Self) -> str\n\nimpl Display for Point:\n\n    def show(self: ref Self) -> str:\n        return format(\"p\")\n";
    assert_has(src, "trait Display {");
    assert_has(src, "fn show(&self) -> String;");
    assert_has(src, "impl Display for Point {");
}

#[test]
fn lexer_and_parse_errors_are_reported_with_location() {
    let err = compile_to_rust("def f(\n", "bad.sn").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("bad.sn"), "error should reference the file: {msg}");
    assert!(msg.contains("error:"), "error should be labelled: {msg}");
}
