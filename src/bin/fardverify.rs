use std::env;
use std::fs;

#[path = "../verify/trace_verify.rs"]
mod trace_verify;

#[path = "../verify/artifact_verify.rs"]
mod artifact_verify;

#[path = "../verify/bundle_verify.rs"]
mod bundle_verify;

fn usage() -> ! {
    eprintln!("usage:");
    eprintln!("  fardverify trace   --out <dir>");
    eprintln!("  fardverify artifact --out <dir>");
    eprintln!("  fardverify bundle  --out <dir>");
    eprintln!("  fardverify chain   --out <dir> [--registry <dir>] [--depth <n>]");
    eprintln!("  fardverify prove   --out <dir> --spec <spec.json>");
    std::process::exit(2);
}

fn get_out(args: &[String]) -> String {
    let mut out: Option<String> = None;
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--out" {
            if i + 1 >= args.len() {
                usage();
            }
            out = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }
    out.unwrap_or_else(|| usage())
}


// ── Chain verification ────────────────────────────────────────────────────────

/// Recursively verify a receipt chain.
/// Returns (total_nodes_verified, max_depth_reached).
fn verify_chain(
    outdir: &str,
    registry: Option<&str>,
    max_depth: usize,
    current_depth: usize,
) -> Result<(usize, usize), String> {
    if current_depth > max_depth {
        return Err(format!("CHAIN_MAX_DEPTH_EXCEEDED {}", max_depth));
    }

    // Verify this node's trace
    trace_verify::verify_trace_outdir(outdir)
        .map_err(|e| format!("node {} trace fail: {}", outdir, e))?;

    let mut total_nodes = 1usize;
    let mut max_d = current_depth;

    // Extract child receipts from this trace
    let trace_path = format!("{}/trace.ndjson", outdir);
    let children = trace_verify::extract_child_receipts(&trace_path)
        .map_err(|e| format!("extract_child_receipts: {}", e))?;

    for (spawn_id, run_digest, result_digest) in &children {
        if run_digest == "sha256:no-trace" {
            // Child had no trace — skip chain verification for this node
            println!("  skip child {} (no-trace)", spawn_id);
            continue;
        }

        // Look up child outdir in registry
        let child_outdir = match registry {
            Some(reg) => {
                // Registry layout: <registry>/<run_digest>/
                let digest_hex = run_digest.trim_start_matches("sha256:");
                format!("{}/{}", reg, digest_hex)
            }
            None => {
                // No registry — skip deep verification
                println!("  skip child {} (no registry)", spawn_id);
                continue;
            }
        };

        // Check child outdir exists
        if !std::path::Path::new(&child_outdir).exists() {
            return Err(format!("CHAIN_MISSING_CHILD spawn_id={} digest={}", spawn_id, run_digest));
        }

        // Verify child's run digest matches what was recorded
        let child_actual_digest = trace_verify::extract_run_digest(&child_outdir)
            .map_err(|e| format!("child digest extract: {}", e))?;
        if &child_actual_digest != run_digest {
            return Err(format!(
                "CHAIN_DIGEST_MISMATCH spawn_id={} expected={} got={}",
                spawn_id, run_digest, child_actual_digest
            ));
        }

        // Verify result digest matches
        let result_path = format!("{}/result.json", child_outdir);
        if std::path::Path::new(&result_path).exists() {
            let result_bytes = fs::read(&result_path)
                .map_err(|e| format!("read result: {}", e))?;
            let actual_result_digest = sha256_hex(&result_bytes);
            if actual_result_digest != *result_digest {
                return Err(format!(
                    "CHAIN_RESULT_MISMATCH spawn_id={} expected={} got={}",
                    spawn_id, result_digest, actual_result_digest
                ));
            }
        }

        println!("  verified child {} depth={}", spawn_id, current_depth + 1);

        // Recurse
        let (child_nodes, child_depth) = verify_chain(
            &child_outdir, registry, max_depth, current_depth + 1
        )?;
        total_nodes += child_nodes;
        max_d = max_d.max(child_depth);
    }

    Ok((total_nodes, max_d))
}

// ── Proof verification ────────────────────────────────────────────────────────

/// A proof spec is a JSON file describing obligations a run must satisfy:
/// {
///   "obligations": [
///     {"type": "result_field", "field": "ok", "expected": true},
///     {"type": "result_field", "field": "count", "min": 1},
///     {"type": "has_child_receipts", "min": 1},
///     {"type": "run_digest_matches", "digest": "sha256:..."},
///     {"type": "no_errors"},
///     {"type": "stdlib_root", "digest": "sha256:..."}
///   ]
/// }
fn verify_proof(outdir: &str, spec_path: &str) -> Result<usize, String> {
    use valuecore::json::{JsonVal, from_slice as json_parse};

    let spec_bytes = fs::read(spec_path)
        .map_err(|e| format!("read spec: {}", e))?;
    let spec: JsonVal = json_parse(&spec_bytes).map(|v| v)
        .map_err(|_| "PROOF_SPEC_PARSE_FAIL".to_string())?;

    let obligations = spec.get("obligations")
        .and_then(|v| v.as_array())
        .ok_or("PROOF_SPEC_MISSING_obligations")?;

    let digests_bytes = fs::read(format!("{}/digests.json", outdir))
        .map_err(|_| "PROOF_MISSING_digests.json")?;
    let digests = json_parse(&digests_bytes)
        .map_err(|_| "PROOF_DIGESTS_PARSE_FAIL")?;

    let result_bytes = fs::read(format!("{}/result.json", outdir)).ok();
    let result: Option<_> = result_bytes.as_ref()
        .and_then(|b| json_parse(b).ok());

    let trace_path = format!("{}/trace.ndjson", outdir);
    let child_receipts = trace_verify::extract_child_receipts(&trace_path)
        .unwrap_or_default();

    let mut satisfied = 0usize;

    for obligation in obligations {
        let obj = obligation.as_object()
            .ok_or("PROOF_OBLIGATION_NOT_OBJECT")?;
        let ob_type = obj.get("type")
            .and_then(|v| v.as_str())
            .ok_or("PROOF_OBLIGATION_MISSING_type")?;

        match ob_type {
            "no_errors" => {
                let ok = digests.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                if !ok {
                    return Err("PROOF_FAIL no_errors: run had errors".to_string());
                }
                satisfied += 1;
            }
            "run_digest_matches" => {
                let expected = obj.get("digest")
                    .and_then(|v| v.as_str())
                    .ok_or("PROOF_OBLIGATION_MISSING_digest")?;
                let actual = trace_verify::extract_run_digest(outdir)
                    .map_err(|e| format!("PROOF_FAIL run_digest_matches: {}", e))?;
                if actual != expected {
                    return Err(format!("PROOF_FAIL run_digest_matches: expected {} got {}", expected, actual));
                }
                satisfied += 1;
            }
            "stdlib_root" => {
                let expected = obj.get("digest")
                    .and_then(|v| v.as_str())
                    .ok_or("PROOF_OBLIGATION_MISSING_digest")?;
                let actual = digests.get("stdlib_root_digest")
                    .and_then(|v| v.as_str())
                    .ok_or("PROOF_FAIL stdlib_root: missing stdlib_root_digest")?;
                if actual != expected {
                    return Err(format!("PROOF_FAIL stdlib_root: expected {} got {}", expected, actual));
                }
                satisfied += 1;
            }
            "has_child_receipts" => {
                let min = obj.get("min")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as usize;
                if child_receipts.len() < min {
                    return Err(format!(
                        "PROOF_FAIL has_child_receipts: need {} got {}",
                        min, child_receipts.len()
                    ));
                }
                satisfied += 1;
            }
            "result_field" => {
                let field = obj.get("field")
                    .and_then(|v| v.as_str())
                    .ok_or("PROOF_OBLIGATION_MISSING_field")?;
                let result = result.as_ref()
                    .ok_or("PROOF_FAIL result_field: no result.json")?;
                // Navigate: result.result.<field>
                let inner = result.get("result").unwrap_or(result);
                let val = inner.get(field)
                    .ok_or_else(|| format!("PROOF_FAIL result_field: field {} not found", field))?;

                // Check expected value
                if let Some(expected) = obj.get("expected") {
                    if val != expected {
                        return Err(format!(
                            "PROOF_FAIL result_field {}: expected {:?} got {:?}",
                            field, expected, val
                        ));
                    }
                }
                // Check min value
                if let Some(min_val) = obj.get("min").and_then(|v| v.as_i64()) {
                    let actual = val.as_i64().ok_or_else(|| format!("PROOF_FAIL result_field {}: not numeric", field))?;
                    if actual < min_val {
                        return Err(format!(
                            "PROOF_FAIL result_field {}: {} < min {}",
                            field, actual, min_val
                        ));
                    }
                }
                // Check max value
                if let Some(max_val) = obj.get("max").and_then(|v| v.as_i64()) {
                    let actual = val.as_i64().ok_or_else(|| format!("PROOF_FAIL result_field {}: not numeric", field))?;
                    if actual > max_val {
                        return Err(format!(
                            "PROOF_FAIL result_field {}: {} > max {}",
                            field, actual, max_val
                        ));
                    }
                }
                satisfied += 1;
            }
            "result_digest" => {
                let expected = obj.get("digest")
                    .and_then(|v| v.as_str())
                    .ok_or("PROOF_OBLIGATION_MISSING_digest")?;
                let result_bytes = fs::read(format!("{}/result.json", outdir))
                    .map_err(|_| "PROOF_FAIL result_digest: no result.json")?;
                let actual = sha256_hex(&result_bytes);
                if actual != expected {
                    return Err(format!("PROOF_FAIL result_digest: expected {} got {}", expected, actual));
                }
                satisfied += 1;
            }
            other => {
                return Err(format!("PROOF_UNKNOWN_OBLIGATION_TYPE {}", other));
            }
        }
    }

    Ok(satisfied)
}

// ── SHA-256 helper ────────────────────────────────────────────────────────────
fn sha256_hex(data: &[u8]) -> String {
    use valuecore::Sha256 as NativeSha256;
    // use sha2::Digest;
    let mut h = NativeSha256::new();
    h.update(data);
    let result = h.finalize();
    let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
    format!("sha256:{}", hex)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        usage();
    }
    let sub = &args[1];
    let outdir = get_out(&args[2..]);

    if sub == "trace" {
        match trace_verify::verify_trace_outdir(&outdir) {
            Ok(()) => {
                let p = format!("{}/PASS_TRACE.txt", outdir);
                let _ = fs::write(&p, b"PASS\n");
                std::process::exit(0);
            }
            Err(e) => {
                let p = format!("{}/FAIL_TRACE.txt", outdir);
                let _ = fs::write(&p, format!("FAIL {}\n", e).as_bytes());
                eprintln!("TRACE_VERIFY_FAIL {}", e);
                std::process::exit(2);
            }
        }
    }

    if sub == "chain" {
        // Walk the receipt chain from a root run outdir
        // Verifies: this run + all child receipts recursively
        let registry = args.windows(2)
            .find(|w| w[0] == "--registry")
            .map(|w| w[1].clone());
        let max_depth: usize = args.windows(2)
            .find(|w| w[0] == "--depth")
            .and_then(|w| w[1].parse().ok())
            .unwrap_or(32);

        match verify_chain(&outdir, registry.as_deref(), max_depth, 0) {
            Ok(stats) => {
                println!("chain ok — {} node(s) verified, depth {}", stats.0, stats.1);
                let p = format!("{}/PASS_CHAIN.txt", outdir);
                let _ = fs::write(&p, format!("PASS nodes={} depth={}
", stats.0, stats.1));
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("CHAIN_VERIFY_FAIL {}", e);
                std::process::exit(2);
            }
        }
    }

    if sub == "prove" {
        // Proof-carrying code: verify a run satisfies a spec
        let spec_path = args.windows(2)
            .find(|w| w[0] == "--spec")
            .map(|w| w[1].clone())
            .unwrap_or_else(|| usage());

        match verify_proof(&outdir, &spec_path) {
            Ok(obligations) => {
                println!("proof ok — {} obligation(s) satisfied", obligations);
                let p = format!("{}/PASS_PROOF.txt", outdir);
                let _ = fs::write(&p, format!("PASS obligations={}
", obligations));
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("PROOF_VERIFY_FAIL {}", e);
                std::process::exit(2);
            }
        }
    }

    if sub == "artifact" {
        match trace_verify::verify_trace_outdir(&outdir) {
            Ok(()) => match artifact_verify::verify_artifact_outdir(&outdir) {
                Ok(()) => {
                    let p = format!("{}/PASS_ARTIFACT.txt", outdir);
                    let _ = fs::write(&p, b"PASS\n");
                    std::process::exit(0);
                }
                Err(e) => {
                    let p = format!("{}/FAIL_ARTIFACT.txt", outdir);
                    let _ = fs::write(&p, format!("FAIL {}\n", e).as_bytes());
                    eprintln!("ARTIFACT_VERIFY_FAIL {}", e);
                    std::process::exit(2);
                }
            },
            Err(e) => {
                let p = format!("{}/FAIL_ARTIFACT.txt", outdir);
                let _ = fs::write(&p, format!("FAIL {}\n", e).as_bytes());
                eprintln!("TRACE_VERIFY_FAIL {}", e);
                std::process::exit(2);
            }
        }
    }

    if sub == "bundle" {
        match bundle_verify::verify_bundle_outdir(&outdir) {
            Ok(()) => {
                let p = format!("{}/PASS_BUNDLE.txt", outdir);
                let _ = fs::write(&p, b"PASS\n");
                std::process::exit(0);
            }
            Err(e) => {
                let p = format!("{}/FAIL_BUNDLE.txt", outdir);
                let _ = fs::write(&p, format!("FAIL {}\n", e).as_bytes());
                eprintln!("BUNDLE_VERIFY_FAIL {}", e);
                std::process::exit(2);
            }
        }
    }

    usage();
}
