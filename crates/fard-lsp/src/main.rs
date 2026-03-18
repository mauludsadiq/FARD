use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug)]
struct FardLsp {
    client: Client,
    docs: Arc<RwLock<HashMap<String, String>>>,
}

async fn publish(client: &Client, uri: Url, text: &str) {
    let errors = fard_v0_5_language_gate::parse_check(text, &uri.to_string());
    let diags: Vec<Diagnostic> = errors.into_iter().map(|(line, col, msg)| {
        Diagnostic {
            range: Range {
                start: Position { line, character: col },
                end:   Position { line, character: col + 80 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            message: msg,
            source: Some("fard-lsp".to_string()),
            ..Default::default()
        }
    }).collect();
    client.publish_diagnostics(uri, diags, None).await;
}

fn stdlib_members(module: &str) -> Option<Vec<(&'static str, &'static str)>> {
    match module {
        "list" => Some(vec![
            ("map", "map(list, fn) -> list"),
            ("filter", "filter(list, fn) -> list"),
            ("fold", "fold(list, init, fn) -> value"),
            ("any", "any(list, fn) -> bool"),
            ("all", "all(list, fn) -> bool"),
            ("find", "find(list, fn) -> value | null"),
            ("find_index", "find_index(list, fn) -> int"),
            ("flat_map", "flat_map(list, fn) -> list"),
            ("take", "take(list, n) -> list"),
            ("drop", "drop(list, n) -> list"),
            ("zip_with", "zip_with(list, list, fn) -> list"),
            ("chunk", "chunk(list, n) -> list"),
            ("sort_by", "sort_by(list, fn) -> list"),
            ("par_map", "par_map(list, fn) -> list"),
            ("len", "len(list) -> int"),
            ("range", "range(start, end) -> list"),
            ("reverse", "reverse(list) -> list"),
            ("concat", "concat(list, list) -> list"),
            ("group_by", "group_by(list, fn) -> map"),
        ]),
        "str" => Some(vec![
            ("len", "len(s) -> int"),
            ("concat", "concat(a, b) -> text"),
            ("join", "join(list, sep) -> text"),
            ("split", "split(s, sep) -> list"),
            ("slice", "slice(s, lo, hi) -> text"),
            ("upper", "upper(s) -> text"),
            ("lower", "lower(s) -> text"),
            ("trim", "trim(s) -> text"),
            ("contains", "contains(s, sub) -> bool"),
            ("starts_with", "starts_with(s, prefix) -> bool"),
            ("ends_with", "ends_with(s, suffix) -> bool"),
            ("pad_left", "pad_left(s, width, char) -> text"),
            ("pad_right", "pad_right(s, width, char) -> text"),
            ("repeat", "repeat(s, n) -> text"),
            ("index_of", "index_of(s, sub) -> int"),
            ("chars", "chars(s) -> list"),
            ("replace", "replace(s, from, to) -> text"),
            ("from_int", "from_int(n) -> text"),
            ("from_float", "from_float(f) -> text"),
        ]),
        "math" => Some(vec![
            ("sin", "sin(x) -> float"),
            ("cos", "cos(x) -> float"),
            ("tan", "tan(x) -> float"),
            ("asin", "asin(x) -> float"),
            ("acos", "acos(x) -> float"),
            ("atan", "atan(x) -> float"),
            ("atan2", "atan2(y, x) -> float"),
            ("log", "log(x) -> float"),
            ("log2", "log2(x) -> float"),
            ("log10", "log10(x) -> float"),
            ("sqrt", "sqrt(x) -> float"),
            ("pow", "pow(base, exp) -> float"),
            ("abs", "abs(x) -> number"),
            ("floor", "floor(x) -> float"),
            ("ceil", "ceil(x) -> float"),
            ("round", "round(x) -> float"),
            ("pi", "pi -> float (3.14159...)"),
            ("e", "e -> float (2.71828...)"),
        ]),
        "io" => Some(vec![
            ("read_file", "read_file(path) -> {t,v} | {t,e}"),
            ("write_file", "write_file(path, content) -> {t,v}"),
            ("append_file", "append_file(path, content) -> {t,v}"),
            ("read_lines", "read_lines(path) -> list"),
            ("read_stdin", "read_stdin() -> text"),
            ("read_stdin_lines", "read_stdin_lines() -> list"),
            ("file_exists", "file_exists(path) -> bool"),
            ("delete_file", "delete_file(path) -> {t,v}"),
            ("list_dir", "list_dir(path) -> list"),
            ("make_dir", "make_dir(path) -> bool"),
        ]),
        "json" => Some(vec![
            ("encode", "encode(val) -> text"),
            ("decode", "decode(text) -> value"),
            ("canonicalize", "canonicalize(text) -> text"),
        ]),
        "map" => Some(vec![
            ("new", "new() -> map"),
            ("get", "get(map, key) -> value"),
            ("set", "set(map, key, value) -> map"),
            ("has", "has(map, key) -> bool"),
            ("delete", "delete(map, key) -> map"),
            ("keys", "keys(map) -> list"),
            ("values", "values(map) -> list"),
            ("entries", "entries(map) -> list"),
        ]),
        "result" => Some(vec![
            ("ok", "ok(v) -> {t:\"ok\", v}"),
            ("err", "err(msg) -> {t:\"err\", e}"),
            ("is_ok", "is_ok(r) -> bool"),
            ("is_err", "is_err(r) -> bool"),
            ("unwrap", "unwrap(r) -> value"),
            ("unwrap_or", "unwrap_or(r, default) -> value"),
            ("map", "map(r, fn) -> result"),
            ("map_err", "map_err(r, fn) -> result"),
            ("and_then", "and_then(r, fn) -> result"),
            ("or_else", "or_else(r, fn) -> result"),
        ]),
        "hash" => Some(vec![
            ("sha256_bytes", "sha256_bytes(bytes) -> digest"),
            ("sha256_text", "sha256_text(text) -> text"),
        ]),
        "http" => Some(vec![
            ("get", "get(url) -> {status, body, headers}"),
            ("post", "post(url, body) -> {status, body, headers}"),
            ("request", "request(rec) -> {status, body, headers}"),
        ]),
        "re" => Some(vec![
            ("is_match", "is_match(pattern, text) -> bool"),
            ("find", "find(pattern, text) -> text | null"),
            ("find_all", "find_all(pattern, text) -> list"),
            ("split", "split(pattern, text) -> list"),
            ("replace", "replace(pattern, text, replacement) -> text"),
        ]),
        "witness" => Some(vec![
            ("verify", "verify(run_id) -> {t,v} | {t,e}"),
            ("verify_chain", "verify_chain(run_id) -> {t,depth} | {t,e}"),
            ("self_digest", "self_digest() -> text (sha256:...)"),
        ]),
        "ffi" => Some(vec![
            ("load", "load(path) -> {t,v} | {t,e}"),
            ("open", "open(path) -> {t,v} | {t,e}"),
            ("call", "call(handle, symbol, args) -> {t,v} | {t,e}"),
            ("call_pure", "call_pure(handle, symbol, args) -> {t,v} | {t,e}"),
            ("call_str", "call_str(handle, symbol, args) -> {t,v} | {t,e}"),
            ("close", "close(handle) -> null"),
        ]),
        "promise" => Some(vec![
            ("spawn", "spawn(fn) -> handle"),
            ("await", "await(handle) -> value"),
        ]),
        "chan" => Some(vec![
            ("new", "new() -> channel"),
            ("send", "send(chan, val) -> bool"),
            ("recv", "recv(chan) -> {t,v} | null"),
            ("try_recv", "try_recv(chan) -> {t,v} | null"),
            ("close", "close(chan) -> bool"),
        ]),
        "mutex" => Some(vec![
            ("new", "new(init) -> mutex"),
            ("lock", "lock(m) -> value"),
            ("unlock", "unlock(m, val) -> bool"),
            ("with_lock", "with_lock(m, fn) -> value"),
        ]),
        "set" => Some(vec![
            ("new", "new() -> set"),
            ("add", "add(set, val) -> set"),
            ("remove", "remove(set, val) -> set"),
            ("has", "has(set, val) -> bool"),
            ("union", "union(set, set) -> set"),
            ("intersect", "intersect(set, set) -> set"),
            ("diff", "diff(set, set) -> set"),
            ("to_list", "to_list(set) -> list"),
            ("from_list", "from_list(list) -> set"),
            ("size", "size(set) -> int"),
        ]),
        "path" => Some(vec![
            ("join", "join(a, b) -> text"),
            ("base", "base(p) -> text"),
            ("dir", "dir(p) -> text"),
            ("ext", "ext(p) -> text"),
            ("isAbs", "isAbs(p) -> bool"),
            ("normalize", "normalize(p) -> text"),
        ]),
        "datetime" => Some(vec![
            ("now", "now() -> int (unix timestamp)"),
            ("format", "format(ts, fmt) -> text"),
            ("parse", "parse(text, fmt) -> int"),
            ("add", "add(ts, unit, n) -> int"),
            ("diff", "diff(a, b) -> int"),
            ("field", "field(ts, field) -> int"),
        ]),
        "process" => Some(vec![
            ("spawn", "spawn(cmd, args, stdin) -> {stdout, stderr, code}"),
            ("exit", "exit(code) -> never"),
        ]),
        "float" => Some(vec![
            ("add", "add(a, b) -> float"),
            ("sub", "sub(a, b) -> float"),
            ("mul", "mul(a, b) -> float"),
            ("div", "div(a, b) -> float"),
            ("sqrt", "sqrt(x) -> float"),
            ("abs", "abs(x) -> float"),
            ("ln", "ln(x) -> float"),
            ("pow", "pow(b, e) -> float"),
            ("neg", "neg(x) -> float"),
            ("from_int", "from_int(n) -> float"),
            ("to_text", "to_text(f) -> text"),
            ("to_str_fixed", "to_str_fixed(f, decimals) -> text"),
            ("is_nan", "is_nan(f) -> bool"),
            ("is_inf", "is_inf(f) -> bool"),
        ]),
        "bigint" => Some(vec![
            ("from_int", "from_int(n) -> bigint"),
            ("from_str", "from_str(s) -> bigint"),
            ("to_str", "to_str(b) -> text"),
            ("add", "add(a, b) -> bigint"),
            ("sub", "sub(a, b) -> bigint"),
            ("mul", "mul(a, b) -> bigint"),
            ("div", "div(a, b) -> bigint"),
            ("mod", "mod(a, b) -> bigint"),
            ("pow", "pow(b, e) -> bigint"),
            ("eq", "eq(a, b) -> bool"),
            ("lt", "lt(a, b) -> bool"),
            ("gt", "gt(a, b) -> bool"),
        ]),
        "bits" => Some(vec![
            ("band", "band(a, b) -> int"),
            ("bor", "bor(a, b) -> int"),
            ("bxor", "bxor(a, b) -> int"),
            ("bnot", "bnot(n) -> int"),
            ("bshl", "bshl(n, bits) -> int"),
            ("bshr", "bshr(n, bits) -> int"),
        ]),
        "linalg" => Some(vec![
            ("zeros", "zeros(rows, cols) -> matrix"),
            ("eye", "eye(n) -> matrix"),
            ("dot", "dot(a, b) -> matrix"),
            ("norm", "norm(v) -> float"),
            ("vec_add", "vec_add(a, b) -> list"),
            ("vec_sub", "vec_sub(a, b) -> list"),
            ("vec_scale", "vec_scale(v, s) -> list"),
            ("transpose", "transpose(m) -> matrix"),
        ]),
        "compress" => Some(vec![
            ("gzip_compress", "gzip_compress(text) -> bytes"),
            ("gzip_decompress", "gzip_decompress(bytes) -> text"),
            ("zstd_compress", "zstd_compress(text) -> bytes"),
            ("zstd_decompress", "zstd_decompress(bytes) -> text"),
        ]),
        "crypto" => Some(vec![
            ("hmac_sha256", "hmac_sha256(key, msg) -> text"),
            ("aes_encrypt", "aes_encrypt(key, plaintext) -> ciphertext"),
            ("aes_decrypt", "aes_decrypt(key, ciphertext) -> plaintext"),
            ("pbkdf2", "pbkdf2(password, salt, iters) -> key"),
        ]),
        "base64" => Some(vec![
            ("encode", "encode(text) -> text"),
            ("decode", "decode(text) -> text"),
        ]),
        "csv" => Some(vec![
            ("parse", "parse(text) -> list of lists"),
            ("encode", "encode(rows) -> text"),
        ]),
        "uuid" => Some(vec![
            ("v4", "v4() -> text (UUID)"),
            ("validate", "validate(s) -> bool"),
        ]),
        "type" => Some(vec![
            ("of", "of(val) -> text"),
            ("is_int", "is_int(v) -> bool"),
            ("is_float", "is_float(v) -> bool"),
            ("is_bool", "is_bool(v) -> bool"),
            ("is_text", "is_text(v) -> bool"),
            ("is_list", "is_list(v) -> bool"),
            ("is_record", "is_record(v) -> bool"),
            ("is_null", "is_null(v) -> bool"),
            ("is_fn", "is_fn(v) -> bool"),
        ]),
        "graph" => Some(vec![
            ("new", "new() -> graph"),
            ("add_node", "add_node(g, id, data) -> graph"),
            ("add_edge", "add_edge(g, from, to, weight) -> graph"),
            ("bfs", "bfs(g, start) -> list"),
            ("dfs", "dfs(g, start) -> list"),
            ("shortest_path", "shortest_path(g, from, to) -> {path, cost}"),
            ("topo_sort", "topo_sort(g) -> list"),
        ]),
        "trace" => Some(vec![
            ("info", "info(msg) -> null"),
            ("warn", "warn(msg) -> null"),
            ("error", "error(msg) -> null"),
            ("span", "span(name, fn) -> value"),
        ]),
        "env" => Some(vec![
            ("get", "get(key) -> text | null"),
            ("args", "args() -> list"),
        ]),
        "ast" => Some(vec![
            ("parse", "parse(source) -> list of AST nodes"),
        ]),
        "eval" => Some(vec![
            ("eval", "eval(source) -> value"),
        ]),
        "net" => Some(vec![
            ("serve", "serve(port, handler_fn) -> never"),
        ]),
        "rec" | "record" => Some(vec![
            ("get", "get(rec, key) -> value"),
            ("set", "set(rec, key, value) -> record"),
            ("has", "has(rec, key) -> bool"),
            ("keys", "keys(rec) -> list"),
            ("merge", "merge(a, b) -> record"),
            ("delete", "delete(rec, key) -> record"),
        ]),
        "option" => Some(vec![
            ("some", "some(v) -> {t:\"some\", v}"),
            ("none", "none -> null"),
            ("is_some", "is_some(o) -> bool"),
            ("is_none", "is_none(o) -> bool"),
            ("unwrap", "unwrap(o) -> value"),
            ("unwrap_or", "unwrap_or(o, default) -> value"),
            ("map", "map(o, fn) -> option"),
            ("and_then", "and_then(o, fn) -> option"),
        ]),
        _ => None,
    }
}

const STDLIB_MODULES: &[&str] = &[
    "std/list", "std/str", "std/math", "std/float", "std/int", "std/bigint",
    "std/bits", "std/map", "std/set", "std/re", "std/json", "std/hash",
    "std/base64", "std/csv", "std/uuid", "std/datetime", "std/path", "std/io",
    "std/http", "std/promise", "std/chan", "std/mutex", "std/ast", "std/eval",
    "std/compress", "std/crypto", "std/graph", "std/type", "std/witness",
    "std/ffi", "std/process", "std/env", "std/net", "std/trace", "std/result",
    "std/option", "std/rec", "std/record", "std/linalg", "std/cell",
    "std/grow", "std/flow", "std/bits", "std/cast",
];

const KEYWORDS: &[&str] = &[
    "let", "fn", "if", "then", "else", "match", "while", "return",
    "import", "export", "artifact", "as", "null", "true", "false", "test",
];

fn hover_for_word(word: &str) -> Option<String> {
    match word {
        "import"   => Some("`import(\"std/list\") as list` -- load a stdlib or package module".to_string()),
        "artifact" => Some("`artifact name = \"sha256:...\"` -- bind a prior verified run by RunID".to_string()),
        "let"      => Some("`let name = expr` -- bind a value in the current scope".to_string()),
        "fn"       => Some("`fn name(params) { body }` -- define a function".to_string()),
        "export"   => Some("`export { name, ... }` -- export names from a module".to_string()),
        "match"    => Some("`match expr { pat => val, _ => default }` -- pattern match".to_string()),
        "if"       => Some("`if cond then expr else expr` -- conditional expression".to_string()),
        "while"    => Some("`while init cond_fn body_fn` -- hash-chained loop with state".to_string()),
        "return"   => Some("`return expr` -- early return from a function".to_string()),
        "null"     => Some("`null` -- the unit value".to_string()),
        "true" | "false" => Some(format!("`{}` -- boolean literal", word)),
        "test"     => Some("`test \"label\" { expr }` -- define a test case".to_string()),
        m => {
            if let Some(members) = stdlib_members(m) {
                let lines: Vec<String> = members.iter()
                    .map(|(name, sig)| format!("- `{}.{}` -- {}", m, name, sig))
                    .collect();
                Some(format!("**std/{}**\n\n{}", m, lines.join("\n")))
            } else {
                None
            }
        }
    }
}

fn word_at(text: &str, line: u32, col: u32) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let l = lines.get(line as usize).copied().unwrap_or("");
    let chars: Vec<char> = l.chars().collect();
    let c = col as usize;
    let start = (0..c).rev()
        .take_while(|&i| chars.get(i).map(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == '/').unwrap_or(false))
        .last().unwrap_or(c);
    let end = (c..chars.len())
        .take_while(|&i| chars.get(i).map(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == '/').unwrap_or(false))
        .last().map(|i| i+1).unwrap_or(c);
    chars[start..end].iter().collect()
}

// Get the module alias at cursor position for dot-completion
// e.g. "list." -> Some("list")
fn module_before_dot(text: &str, line: u32, col: u32) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let l = lines.get(line as usize)?;
    let chars: Vec<char> = l.chars().collect();
    let c = col as usize;
    if c == 0 { return None; }
    // Check if char before cursor is '.'
    if chars.get(c.saturating_sub(1)) != Some(&'.') { return None; }
    // Walk back to find the identifier before the dot
    let dot_pos = c - 1;
    let start = (0..dot_pos).rev()
        .take_while(|&i| chars.get(i).map(|ch| ch.is_alphanumeric() || *ch == '_').unwrap_or(false))
        .last().unwrap_or(dot_pos);
    let word: String = chars[start..dot_pos].iter().collect();
    if word.is_empty() { None } else { Some(word) }
}

// Find what module name an alias maps to by scanning import statements
fn resolve_alias(text: &str, alias: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        // import("std/list") as list
        if trimmed.starts_with("import(") && trimmed.contains(&format!("as {}", alias)) {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start+1..].find('"') {
                    let path = &trimmed[start+1..start+1+end];
                    // Extract module name from path: "std/list" -> "list"
                    let module = path.split('/').last().unwrap_or(path);
                    return Some(module.to_string());
                }
            }
        }
    }
    None
}

fn make_completion_item(label: &str, detail: &str, kind: CompletionItemKind) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(kind),
        detail: Some(detail.to_string()),
        insert_text: Some(label.to_string()),
        ..Default::default()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for FardLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), "\"".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "fard-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "fard-lsp initialized").await;
    }

    async fn shutdown(&self) -> Result<()> { Ok(()) }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.docs.write().await.insert(uri.to_string(), text.clone());
        publish(&self.client, uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            self.docs.write().await.insert(uri.to_string(), change.text.clone());
            publish(&self.client, uri, &change.text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(text) = params.text {
            self.docs.write().await.insert(uri.to_string(), text.clone());
            publish(&self.client, uri, &text).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let pos = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri;
        let docs = self.docs.read().await;
        if let Some(text) = docs.get(&uri.to_string()) {
            let word = word_at(text, pos.line, pos.character);
            if let Some(doc) = hover_for_word(&word) {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc,
                    }),
                    range: None,
                }));
            }
        }
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let pos = params.text_document_position.position;
        let uri = params.text_document_position.text_document.uri;
        let docs = self.docs.read().await;
        let text = match docs.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        drop(docs);

        let mut items: Vec<CompletionItem> = Vec::new();

        // Check if we're doing dot-completion (e.g. "list.")
        if let Some(alias) = module_before_dot(&text, pos.line, pos.character) {
            // Resolve alias to module name
            let module = resolve_alias(&text, &alias)
                .unwrap_or_else(|| alias.clone());

            if let Some(members) = stdlib_members(&module) {
                for (name, sig) in members {
                    items.push(make_completion_item(
                        name,
                        sig,
                        CompletionItemKind::METHOD,
                    ));
                }
                return Ok(Some(CompletionResponse::Array(items)));
            }
        }

        // Check if we're inside an import string: import("std/
        let lines: Vec<&str> = text.lines().collect();
        let current_line = lines.get(pos.line as usize).copied().unwrap_or("");
        let col = pos.character as usize;
        let before_cursor = &current_line[..col.min(current_line.len())];

        if before_cursor.contains("import(\"") || before_cursor.contains("import('") {
            for module in STDLIB_MODULES {
                items.push(make_completion_item(
                    module,
                    &format!("import(\"{}\") as ...", module),
                    CompletionItemKind::MODULE,
                ));
            }
            return Ok(Some(CompletionResponse::Array(items)));
        }

        // Default: keywords + stdlib module aliases from current file
        for kw in KEYWORDS {
            items.push(make_completion_item(kw, "keyword", CompletionItemKind::KEYWORD));
        }

        // Add imported aliases as completions
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import(") {
                if let Some(as_pos) = trimmed.find(" as ") {
                    let alias = trimmed[as_pos+4..].trim();
                    if !alias.is_empty() {
                        items.push(make_completion_item(
                            alias,
                            &format!("imported as {}", alias),
                            CompletionItemKind::MODULE,
                        ));
                    }
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    
    }
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.clone();
        let pos = params.text_document_position_params.position;
        let docs = self.docs.read().await;
        let text = match docs.get(uri.as_str()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        let word = word_at_position(&text, pos.line, pos.character);
        if word.is_empty() { return Ok(None); }

        // Find the definition of `word` in the document
        if let Some(loc) = find_definition(&text, &word, &uri) {
            return Ok(Some(GotoDefinitionResponse::Scalar(loc)));
        }
        Ok(None)
    }

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let pos = params.text_document_position.position;
        let docs = self.docs.read().await;
        let text = match docs.get(uri.as_str()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        let word = word_at_position(&text, pos.line, pos.character);
        if word.is_empty() { return Ok(None); }

        let locs = find_references(&text, &word, &uri);
        if locs.is_empty() { return Ok(None); }
        Ok(Some(locs))
    }

}


// ── Symbol navigation helpers ─────────────────────────────────────────────────

/// Extract the word (identifier) at a given position in the text.
fn word_at_position(text: &str, line: u32, character: u32) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = line as usize;
    if line_idx >= lines.len() { return String::new(); }
    let line_str = lines[line_idx];
    let chars: Vec<char> = line_str.chars().collect();
    let col = character as usize;
    if col >= chars.len() { return String::new(); }

    // Extend left
    let mut start = col;
    while start > 0 && (chars[start-1].is_alphanumeric() || chars[start-1] == '_') {
        start -= 1;
    }
    // Extend right
    let mut end = col;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    chars[start..end].iter().collect()
}

/// Find the definition location of a symbol in the document.
/// Looks for: `fn name(`, `let name =`, `fn name `, param names.
fn find_definition(text: &str, name: &str, uri: &Url) -> Option<Location> {
    let patterns = [
        format!("fn {}(", name),
        format!("fn {} (", name),
        format!("let {} =", name),
        format!("let {} ", name),
    ];

    for (line_idx, line) in text.lines().enumerate() {
        for pat in &patterns {
            if let Some(col) = line.find(pat.as_str()) {
                // Find where the name starts within the pattern
                let name_offset = pat.find(name).unwrap_or(0);
                let char_col = col + name_offset;
                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: line_idx as u32, character: char_col as u32 },
                        end:   Position { line: line_idx as u32, character: (char_col + name.len()) as u32 },
                    },
                });
            }
        }
        // Check fn params: `fn foo(name, ` or `fn foo(a, name)`
        if line.trim_start().starts_with("fn ") {
            if let Some(paren_start) = line.find('(') {
                if let Some(paren_end) = line.find(')') {
                    let params_str = &line[paren_start+1..paren_end];
                    let mut col_offset = paren_start + 1;
                    for param in params_str.split(',') {
                        let trimmed = param.trim();
                        // Handle "param: type" or just "param"
                        let pname = trimmed.split(':').next().unwrap_or("").trim();
                        if pname == name {
                            let char_col = col_offset + param.len() - param.trim_start().len();
                            return Some(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position { line: line_idx as u32, character: char_col as u32 },
                                    end:   Position { line: line_idx as u32, character: (char_col + name.len()) as u32 },
                                },
                            });
                        }
                        col_offset += param.len() + 1; // +1 for comma
                    }
                }
            }
        }
    }
    None
}

/// Find all references to a symbol in the document.
fn find_references(text: &str, name: &str, uri: &Url) -> Vec<Location> {
    let mut locs = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        let mut search_from = 0;
        while let Some(col) = line[search_from..].find(name) {
            let abs_col = search_from + col;
            // Check word boundaries — must not be preceded/followed by alphanumeric or _
            let before_ok = abs_col == 0 || {
                let c = line.as_bytes()[abs_col - 1];
                !c.is_ascii_alphanumeric() && c != b'_'
            };
            let after_ok = abs_col + name.len() >= line.len() || {
                let c = line.as_bytes()[abs_col + name.len()];
                !c.is_ascii_alphanumeric() && c != b'_'
            };
            if before_ok && after_ok {
                locs.push(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: line_idx as u32, character: abs_col as u32 },
                        end:   Position { line: line_idx as u32, character: (abs_col + name.len()) as u32 },
                    },
                });
            }
            search_from = abs_col + name.len();
            if search_from >= line.len() { break; }
        }
    }
    locs
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| FardLsp {
        client,
        docs: Arc::new(RwLock::new(HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod nav_tests {
    use super::*;
    use tower_lsp::lsp_types::Url;

    fn test_uri() -> Url {
        Url::parse("file:///tmp/test.fard").unwrap()
    }

    const SRC: &str = "\
fn double(n) { n * 2 }
fn triple(n) { n * 3 }
fn compute(x) {
  let a = double(x)
  let b = triple(x)
  a + b
}
let result = compute(10)";

    #[test]
    fn test_word_at_position() {
        assert_eq!(word_at_position(SRC, 0, 3), "double");
        assert_eq!(word_at_position(SRC, 0, 10), "n");
        assert_eq!(word_at_position(SRC, 3, 6), "a");
    }

    #[test]
    fn test_find_definition_fn() {
        let loc = find_definition(SRC, "double", &test_uri()).unwrap();
        assert_eq!(loc.range.start.line, 0);
        assert_eq!(loc.range.start.character, 3);
    }

    #[test]
    fn test_find_definition_let() {
        let loc = find_definition(SRC, "result", &test_uri()).unwrap();
        assert_eq!(loc.range.start.line, 7);
    }

    #[test]
    fn test_find_definition_param() {
        let loc = find_definition(SRC, "x", &test_uri()).unwrap();
        // x is a param of compute on line 2
        assert_eq!(loc.range.start.line, 2);
    }

    #[test]
    fn test_find_references() {
        let refs = find_references(SRC, "double", &test_uri());
        // double appears on line 0 (def) and line 3 (call)
        assert!(refs.len() >= 2);
        assert!(refs.iter().any(|r| r.range.start.line == 0));
        assert!(refs.iter().any(|r| r.range.start.line == 3));
    }

    #[test]
    fn test_references_word_boundary() {
        // "double" should not match inside "double_check"
        let src = "fn double_check(n) { n }\nlet x = double(1)";
        let refs = find_references(src, "double", &test_uri());
        // Only the call on line 1, not the fn name "double_check"
        assert!(refs.iter().all(|r| {
            let line = src.lines().nth(r.range.start.line as usize).unwrap_or("");
            !line[r.range.start.character as usize..].starts_with("double_")
        }));
    }
}
