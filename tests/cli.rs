use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_multiplication-product-validator"))
}

#[test]
fn prints_pairs_for_twelve() {
    let output = bin().args(["12", "--no-csv"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout,
        "Found 3 multiplications for the product 12.\n\
         1 * 12 = 12\n\
         2 * 6 = 12\n\
         3 * 4 = 12\n"
    );
}

#[test]
fn rejects_zero_with_a_failure_exit_code() {
    let output = bin().arg("0").output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("positive integer"), "{stderr}");
}

#[test]
fn writes_the_csv_file() {
    let dir = std::env::temp_dir().join(format!("mpv-cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let output = bin()
        .current_dir(&dir)
        .args(["36", "--name", "out"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let csv = std::fs::read_to_string(dir.join("out.csv")).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(
        csv,
        "Multiplier,Multiplicand,Product\n\
         1,36,36\n2,18,36\n3,12,36\n4,9,36\n6,6,36\n"
    );
}
