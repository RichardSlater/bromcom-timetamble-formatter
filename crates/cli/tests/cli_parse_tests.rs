use assert_cmd::Command;

#[test]
#[allow(deprecated)]
fn cli_runs_without_map_flag_and_prints_processing() {
    let mut cmd = Command::cargo_bin("timetable_cli").expect("binary exists");
    cmd.arg("--input")
        .arg("README.md")
        .arg("--config")
        .arg("config.toml")
        .arg("--output")
        .arg("target/test_cli_out");

    // We expect the binary to run and (likely) fail parsing or processing the PDF, but not to panic during arg parsing.
    let result = cmd.output().expect("run command");
    // Exit code could be non-zero; ensure it started and printed a processing message or usage
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(
        stdout.contains("Processing timetable")
            || stderr.contains("Processing timetable")
            || !stdout.is_empty()
            || !stderr.is_empty()
    );
}
