use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/sunrise-hdr.jxr")
}

fn unique_output_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX_EPOCH")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "jxr2uhdr-e2e-{}-{timestamp}.jpg",
        std::process::id()
    ))
}

#[test]
fn cli_converts_sample_jxr_to_ultra_hdr_jpeg() {
    let input = fixture_path();
    let output = unique_output_path();

    let run = Command::new(env!("CARGO_BIN_EXE_jxr2uhdr"))
        .arg("--input")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("CLI binary should run");

    if !run.status.success() {
        panic!(
            "CLI exited with {:?}\nstdout:\n{}\nstderr:\n{}",
            run.status.code(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let encoded = fs::read(&output).expect("output JPEG should be written");

    assert!(encoded.len() > 1024, "encoded JPEG should not be tiny");
    assert_eq!(&encoded[..2], &[0xFF, 0xD8], "output should be a JPEG");
    assert!(
        encoded
            .windows(b"urn:iso:std:iso:ts:21496:-1".len())
            .any(|window| window == b"urn:iso:std:iso:ts:21496:-1"),
        "output should contain the Ultra HDR ISO marker"
    );

    fs::remove_file(&output).expect("temporary output should be removable");
}
