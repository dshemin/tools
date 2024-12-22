use assert_cmd::prelude::*;
use file_diff::diff;
use std::env;
use std::process::Command;

#[test]
fn success_empty() -> Result<(), Box<dyn std::error::Error>> {
    test_case_success("empty.xlsx", "empty.csv")
}

#[test]
fn success_filled() -> Result<(), Box<dyn std::error::Error>> {
    test_case_success("filled.xlsx", "filled.csv")
}

fn test_case_success(
    in_filename: &str,
    expected_out_filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("potok_to_snowball")?;

    let path = env::current_dir()?.join("tests").join("data");

    let out_path = env::temp_dir().join(expected_out_filename);

    cmd.arg("transform")
        .arg("--in-path")
        .arg(path.clone().join(in_filename))
        .arg("--out-path")
        .arg(out_path.clone());

    cmd.assert().success();

    assert!(diff(
        path.clone().join(expected_out_filename).to_str().unwrap(),
        out_path.to_str().unwrap(),
    ));

    Ok(())
}
