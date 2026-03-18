use std::collections::BTreeSet;
use valuecore::json::{JsonVal, from_slice, from_str as json_from_str, escape_string};
use std::fs;

fn is_sha256(s: &str) -> bool {
    if !s.starts_with("sha256:") {
        return false;
    }
    let h = &s[7..];
    if h.len() != 64 {
        return false;
    }
    h.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

fn canon_line(v: &JsonVal) -> Result<String, String> {
    fn canon_value(v: &JsonVal, out: &mut String) -> Result<(), String> {
        match v {
            JsonVal::Null => {
                out.push_str("null");
                Ok(())
            }
            JsonVal::Bool(b) => {
                out.push_str(if *b { "true" } else { "false" });
                Ok(())
            }
            JsonVal::Float(f) => {
                let s = format!("{}", f);
                if s.contains('+') { return Err("CANON_NUM_PLUS".into()); }
                if s.ends_with(".0") { return Err("CANON_NUM_DOT0".into()); }
                out.push_str(&s);
                Ok(())
            }
            JsonVal::Int(n) => {
                let s = n.to_string();
                if s.contains('+') {
                    return Err("CANON_NUM_PLUS".into());
                }
                if s.starts_with('0') && s.len() > 1 && !s.starts_with("0.") {
                    return Err("CANON_NUM_LEADING_ZERO".into());
                }
                if s.ends_with(".0") {
                    return Err("CANON_NUM_DOT0".into());
                }
                out.push_str(&s);
                Ok(())
            }
            JsonVal::Str(s) => {
                out.push_str(&escape_string(s));
                Ok(())
            }
            JsonVal::Array(a) => {
                out.push('[');
                for (i, x) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    canon_value(x, out)?;
                }
                out.push(']');
                Ok(())
            }
            JsonVal::Object(m) => {
                let mut keys: Vec<&String> = m.keys().collect();
                keys.sort();
                out.push('{');
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&escape_string(k));
                    out.push(':');
                    canon_value(&m[*k], out)?;
                }
                out.push('}');
                Ok(())
            }
        }
    }

    let mut out = String::new();
    canon_value(v, &mut out)?;
    Ok(out)
}

fn expect_only_keys(
    obj: &std::collections::BTreeMap<String, JsonVal>,
    allowed: &[&str],
) -> Result<(), String> {
    let allow: BTreeSet<&str> = allowed.iter().copied().collect();
    for k in obj.keys() {
        if !allow.contains(k.as_str()) {
            return Err(format!("TRACE_EXTRA_KEY {}", k));
        }
    }
    Ok(())
}

fn expect_str<'a>(
    obj: &'a std::collections::BTreeMap<String, JsonVal>,
    k: &str,
) -> Result<&'a str, String> {
    obj.get(k)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("TRACE_EXPECT_STRING {}", k))
}

fn expect_t(obj: &std::collections::BTreeMap<String, JsonVal>) -> Result<&str, String> {
    expect_str(obj, "t")
}

pub fn verify_trace_outdir(outdir: &str) -> Result<(), String> {
    let digests_p = format!("{}/digests.json", outdir);
    let trace_p = format!("{}/trace.ndjson", outdir);
    let digests_bytes = fs::read(&digests_p).map_err(|_| "M2_MISSING_digests.json".to_string())?;
    let digests_v: JsonVal =
        from_slice(&digests_bytes).map_err(|_| "M2_DIGESTS_PARSE_FAIL".to_string())?;
    let ok = digests_v
        .get("ok")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "M2_DIGESTS_MISSING_ok".to_string())?;

    let trace_bytes = fs::read(&trace_p).map_err(|_| "M2_MISSING_trace.ndjson".to_string())?;
    let trace_str =
        std::str::from_utf8(&trace_bytes).map_err(|_| "M2_TRACE_NOT_UTF8".to_string())?;

    let allowed_t: BTreeSet<&str> = [
        "module_resolve",
        "module_graph",
        "artifact_in",
        "artifact_in_named",
        "artifact_out",
        "artifact_dep",
        "artifact_derived",
        "error",
        // Witnessed concurrency
        "child_spawn",
        "child_receipt",
        // Tracing
        "emit",
        "grow_node",
        "trace_info",
        "trace_warn",
        "trace_error",
        "trace_span",
        // Witness
        "witness_verify",
        "witnessed_failure",
        "while_start",
        "while_step",
        "while_end",
        "ffi_oracle",
        "ffi_checked",
        "spawn_ordered_complete",
    ]
    .into_iter()
    .collect();

    let mut saw_non_module_resolve = false;
    let mut module_graph_count = 0usize;
    let mut error_count = 0usize;
    let mut last_t: Option<String> = None;

    for (idx, raw_line) in trace_str.split('\n').enumerate() {
        if raw_line.is_empty() {
            if idx + 1 == trace_str.split('\n').count() {
                break;
            }
            return Err("M2_EMPTY_LINE".into());
        }
        if raw_line.ends_with(' ') || raw_line.ends_with('\t') || raw_line.contains("\r") {
            return Err("M2_TRAILING_SPACE_OR_CR".into());
        }

        let v: JsonVal =
            json_from_str(raw_line).map_err(|_| "M2_TRACE_LINE_PARSE_FAIL".to_string())?;
        let obj = v
            .as_object()
            .ok_or_else(|| "M2_TRACE_LINE_NOT_OBJECT".to_string())?;

        let t = expect_t(obj)?;
        if !allowed_t.contains(t) {
            return Err(format!("M2_BAD_EVENT_TAG {}", t));
        }

        let canon = canon_line(&v)?;
        // Canon check: only enforce for structural events, not informational ones
        let t_for_canon = obj.get("t").and_then(|v| v.as_str()).unwrap_or("");
        let skip_canon = matches!(t_for_canon,
            "emit" | "grow_node" | "trace_info" | "trace_warn" |
            "trace_error" | "trace_span" | "witness_verify" | "artifact_derived"
        );
        if !skip_canon && canon != raw_line {
            return Err(format!(
                "M2_CANON_MISMATCH idx={} raw={} canon={}",
                idx, raw_line, canon
            )
            .into());
        }

        match t {
            "module_resolve" => {
                expect_only_keys(obj, &["cid", "kind", "name", "t"])?;
                let cid = expect_str(obj, "cid")?;
                if !is_sha256(cid) {
                    return Err("M2_BAD_CID".into());
                }
                let kind = expect_str(obj, "kind")?;
                let _name = expect_str(obj, "name")?;
                if !matches!(kind, "std" | "rel" | "abs" | "vendor") {
                    return Err("M2_BAD_KIND".into());
                }
                if saw_non_module_resolve {
                    return Err("M2_ORDER_MODULE_RESOLVE_PREFIX".into());
                }
            }
            "module_graph" => {
                expect_only_keys(obj, &["cid", "t"])?;
                let cid = expect_str(obj, "cid")?;
                if !is_sha256(cid) {
                    return Err("M2_BAD_CID".into());
                }
                module_graph_count += 1;
                saw_non_module_resolve = true;
            }
            "artifact_in" | "artifact_out" => {
                expect_only_keys(obj, &["cid", "name", "t"])?;
                let cid = expect_str(obj, "cid")?;
                if !is_sha256(cid) {
                    return Err("M2_BAD_CID".into());
                }
                let _name = expect_str(obj, "name")?;
                saw_non_module_resolve = true;
            }
            "error" => {
                expect_only_keys(obj, &["code", "e", "message", "t"])?;
                let _code = expect_str(obj, "code")?;
                let _msg = expect_str(obj, "message")?;
                error_count += 1;
                saw_non_module_resolve = true;
            }
            "child_spawn" => {
                // {t: "child_spawn", spawn_id: "spawn_<uuid>"}
                let _spawn_id = expect_str(obj, "spawn_id")?;
                saw_non_module_resolve = true;
            }
            "child_receipt" => {
                // {t: "child_receipt", spawn_id: "...", run_digest: "sha256:...", result_digest: "sha256:..."}
                let _spawn_id = expect_str(obj, "spawn_id")?;
                let run_digest = expect_str(obj, "run_digest")?;
                let result_digest = expect_str(obj, "result_digest")?;
                if !is_sha256(run_digest) && run_digest != "sha256:no-trace" {
                    return Err(format!("M2_BAD_RUN_DIGEST {}", run_digest));
                }
                if !is_sha256(result_digest) && result_digest != "sha256:no-result" {
                    return Err(format!("M2_BAD_RESULT_DIGEST {}", result_digest));
                }
                saw_non_module_resolve = true;
            }
            "artifact_in_named" => {
                let cid = expect_str(obj, "cid")?;
                if !is_sha256(cid) { return Err("M2_BAD_CID".into()); }
                saw_non_module_resolve = true;
            }
            "artifact_dep" => {
                let run_id = expect_str(obj, "run_id")?;
                if !is_sha256(run_id) { return Err(format!("M2_BAD_RUN_ID {}", run_id)); }
                saw_non_module_resolve = true;
            }
            "artifact_derived" | "emit" | "grow_node" |
            "trace_info" | "trace_warn" | "trace_error" | "trace_span" |
            "witness_verify" | "witnessed_failure" |
            "while_start" | "while_step" | "while_end" |
            "ffi_oracle" | "ffi_checked" | "spawn_ordered_complete" => {
                // Informational events — no strict schema, just require "t" field
                saw_non_module_resolve = true;
            }
            _ => return Err("M2_UNREACHABLE".into()),
        }

        last_t = Some(t.to_string());
    }

    if module_graph_count != 1 {
        return Err("M2_MODULE_GRAPH_NOT_ONCE".into());
    }

    if error_count > 0 {
        if error_count != 1 {
            return Err("M2_ERROR_NOT_ONCE".into());
        }
        if last_t.as_deref() != Some("error") {
            return Err("M2_ERROR_NOT_LAST".into());
        }
    }

    let result_p = format!("{}/result.json", outdir);
    let error_p = format!("{}/error.json", outdir);

    let has_result = fs::metadata(&result_p).is_ok();
    let has_error = fs::metadata(&error_p).is_ok();

    if ok {
        if error_count != 0 {
            return Err("M2_OK_MUST_HAVE_NO_ERROR_EVENT".into());
        }
        if !has_result {
            return Err("M2_OK_MUST_HAVE_result.json".into());
        }
        if has_error {
            return Err("M2_OK_MUST_NOT_HAVE_error.json".into());
        }
    } else {
        if error_count != 1 {
            return Err("M2_FAIL_MUST_HAVE_ONE_ERROR_EVENT".into());
        }
        if has_result {
            return Err("M2_FAIL_MUST_NOT_HAVE_result.json".into());
        }
        if !has_error {
            return Err("M2_FAIL_MUST_HAVE_error.json".into());
        }
    }

    Ok(())
}

/// Extract all child_receipt entries from a trace
pub fn extract_child_receipts(trace_path: &str) -> Result<Vec<(String, String, String)>, String> {
    let bytes = std::fs::read(trace_path).map_err(|e| format!("IO: {e}"))?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "UTF8".to_string())?;
    let mut receipts = Vec::new();
    for line in text.split('\n') {
        if line.is_empty() { continue; }
        if let Ok(v) = valuecore::json::from_str(line) {
            if let Some(obj) = v.as_object() {
                if obj.get("t").and_then(|t| t.as_str()) == Some("child_receipt") {
                    let spawn_id = obj.get("spawn_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let run_digest = obj.get("run_digest").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let result_digest = obj.get("result_digest").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    receipts.push((spawn_id, run_digest, result_digest));
                }
            }
        }
    }
    Ok(receipts)
}

/// Extract the run digest from a digests.json
pub fn extract_run_digest(outdir: &str) -> Result<String, String> {
    let path = format!("{}/digests.json", outdir);
    let bytes = std::fs::read(&path).map_err(|e| format!("IO: {e}"))?;
    let v: valuecore::json::JsonVal = valuecore::json::from_slice(&bytes)
        .map_err(|_| "PARSE".to_string())?;
    v.get("preimage_sha256")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "MISSING_preimage_sha256".to_string())
}
