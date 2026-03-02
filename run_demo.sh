#!/usr/bin/env bash

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_RUN="$ROOT/target/debug/fardrun"
BIN_FARD="$ROOT/target/debug/fard"
MANIFEST="$ROOT/tests/conformance/frozen_digests_v1.txt"
TMP_ACTUAL="$(mktemp)"
MODE="${1:-check}"
PASS=0
FAIL=0

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}

sha256_text() {
  local tmp
  tmp="$(mktemp)"
  cat > "$tmp"
  sha256_file "$tmp"
  rm -f "$tmp"
}

hash_dir_tree() {
  local dir="$1"
  local files
  files="$(find "$dir" -type f | LC_ALL=C sort)"
  if [ -z "$files" ]; then
    printf '%s' "EMPTY_DIRECTORY" | sha256_text
    return
  fi
  while IFS= read -r f; do
    rel="${f#$dir/}"
    printf '%s  %s\n' "$rel" "$(sha256_file "$f")"
  done <<EOF2 | sha256_text
$files
EOF2
}

record_line() {
  local engine="$1"
  local name="$2"
  local prog="$3"
  local digest="$4"
  printf '%s|%s|%s|%s\n' "$engine" "$name" "$prog" "$digest" >> "$TMP_ACTUAL"
}

run_fard() {
  local name="$1"
  local prog="$2"
  local output
  local digest

  if output="$("$BIN_FARD" run "$ROOT/$prog" 2>&1)"; then
    digest="$(printf '%s' "$output" | sha256_text)"
    echo "PASS  $name  $digest"
    PASS=$((PASS+1))
    record_line "fard" "$name" "$prog" "$digest"
  else
    echo "FAIL  $name"
    FAIL=$((FAIL+1))
  fi
}

run_fardrun() {
  local name="$1"
  local prog="$2"
  local out
  local digest

  out="$(mktemp -d "/tmp/fard_conformance_XXXXXX")"

  if "$BIN_RUN" run --program "$ROOT/$prog" --out "$out" >/dev/null 2>&1; then
    if [ -f "$out/error.json" ]; then
      echo "FAIL  $name"
      FAIL=$((FAIL+1))
    else
      digest="$(hash_dir_tree "$out")"
      echo "PASS  $name  $digest"
      PASS=$((PASS+1))
      record_line "fardrun" "$name" "$prog" "$digest"
    fi
  else
    echo "FAIL  $name"
    FAIL=$((FAIL+1))
  fi

  rm -rf "$out"
}

printf '%s\n' "FARD v0.5 - Digest Conformance Suite"
printf '%s\n' "===================================="

run_fard    "mathematical_proof_system"   "examples/mathematical_proof_system/main.fard"
run_fard    "collapse_chess_z"            "examples/collapse_chess_z/main.fard"
run_fard    "collapse_structural_numbers" "examples/collapse_structural_numbers/main.fard"

run_fardrun "qasim_safety"                "examples/qasim_safety/qasim_safety.fard"
run_fardrun "collapse_periodic_table"     "examples/collapse_periodic_table/collapse_periodic_table.fard"
run_fardrun "collapse_coin_canonicalize"  "examples/collapse_coin/canonicalize_tx.fard"
run_fardrun "collapse_coin_rewards"       "examples/collapse_coin/compute_rewards.fard"
run_fardrun "collapse_coin_settle"        "examples/collapse_coin/settle.fard"
run_fardrun "collapse_coin_verify_jwt"    "examples/collapse_coin/verify_jwt.fard"
run_fardrun "collapse_stack_apply_delta"  "examples/collapse_stack/apply_delta.fard"
run_fardrun "collapse_stack_verify_z"     "examples/collapse_stack/verify_zstate.fard"
run_fardrun "sembit_verify"               "examples/sembit/sembit_verify.fard"
run_fardrun "kitchen_sink"                "examples/kitchen_sink_v0_5.fard"

LC_ALL=C sort "$TMP_ACTUAL" -o "$TMP_ACTUAL"

if [ "$MODE" = "--record" ] || [ "$MODE" = "record" ]; then
  if [ "$FAIL" -eq 0 ]; then
    cp "$TMP_ACTUAL" "$MANIFEST"
    printf '%s\n' "------------------------------------"
    printf '%s\n' "RECORDED  $MANIFEST"
  else
    printf '%s\n' "------------------------------------"
    printf '%s\n' "DID NOT RECORD BASELINE (suite has failures)"
  fi
else
  if [ ! -f "$MANIFEST" ]; then
    printf '%s\n' "FAIL  frozen manifest missing: $MANIFEST"
    FAIL=$((FAIL+1))
  else
    if diff -u "$MANIFEST" "$TMP_ACTUAL"; then
      printf '%s\n' "DIGESTS MATCH"
    else
      printf '%s\n' "DIGEST MISMATCH"
      FAIL=$((FAIL+1))
    fi
  fi
fi

rm -f "$TMP_ACTUAL"

printf '%s\n' "===================================="
printf '%s\n' "  $PASS passed  $FAIL failed"

[ "$FAIL" -eq 0 ]
