use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn read_json(path: &std::path::Path) -> serde_json::Value {
    let s = fs::read_to_string(path).unwrap();
    serde_json::from_str(&s).unwrap()
}

/// Removes the directory on drop, including when the test panics.
struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn png_red_1x1_bytes_are_canonical_and_exact() {
    let exe = env!("CARGO_BIN_EXE_fardrun");
    let out_dir = std::env::temp_dir().join(format!("fard_png_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&out_dir);
    let _guard = TempDirGuard(out_dir.clone());

    let status = Command::new(exe)
        .args([
            "run",
            "--program",
            "tests/lang_gates_v1/programs/g99_png_red_1x1.fard",
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .status()
        .unwrap();

    assert!(status.success(), "runner failed");

    let v = read_json(&out_dir.join("result.json"));

    let expect_hex = "89504e470d0a1a0a0000000d4948445200000001000000010802000000907753de0000000f494441547801010400fbff00ff0000030101008d1de5820000000049454e44ae426082";
    let expect = serde_json::json!({
        "t":"bytes",
        "v": format!("hex:{}", expect_hex)
    });

    assert_eq!(v["result"], expect, "result.json mismatch");
}
