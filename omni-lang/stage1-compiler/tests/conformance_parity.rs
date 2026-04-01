use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy)]
struct Case {
    input_rel: &'static str,
    stage1_should_pass: bool,
    omnc_should_pass: bool,
}

fn bin_with_ext(base: &Path, stem: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        return base.join(format!("{stem}.exe"));
    }
    #[cfg(not(target_os = "windows"))]
    {
        base.join(stem)
    }
}

fn repo_root() -> PathBuf {
    let omni_lang = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("stage1-compiler should be inside omni-lang");
    omni_lang
        .parent()
        .expect("omni-lang should be inside repository")
        .to_path_buf()
}

fn run_stage1_check(stage1_bin: &Path, input: &Path) -> bool {
    Command::new(stage1_bin)
        .arg("check")
        .arg(input)
        .status()
        .expect("failed to execute omni_stage1")
        .success()
}

fn run_omnc_emit(omnc_bin: &Path, input: &Path) -> bool {
    let out = std::env::temp_dir().join("omni_parity_test.ovm");
    Command::new(omnc_bin)
        .arg(input)
        .arg("-o")
        .arg(out)
        .status()
        .expect("failed to execute omnc")
        .success()
}

#[test]
fn conformance_matrix_stays_stable() {
    let root = repo_root();
    let compiler_dir = root.join("omni-lang").join("compiler");

    let stage1_bin = bin_with_ext(&compiler_dir.join("target").join("release"), "omni_stage1");
    let omnc_bin = bin_with_ext(&compiler_dir.join("target").join("release"), "omnc");

    assert!(
        stage1_bin.exists(),
        "missing stage1 binary at {}. build with cargo build --release in omni-lang/compiler",
        stage1_bin.display()
    );
    assert!(
        omnc_bin.exists(),
        "missing omnc binary at {}. build with cargo build --release in omni-lang/compiler",
        omnc_bin.display()
    );

    let cases = [
        // Stage1 corpus (brace/semicolon syntax)
        Case {
            input_rel: "examples/arithmetic.omni",
            stage1_should_pass: true,
            omnc_should_pass: true,
        },
        Case {
            input_rel: "examples/fibonacci.omni",
            stage1_should_pass: true,
            omnc_should_pass: true,
        },
        // Mainline corpus (current omnc syntax)
        Case {
            input_rel: "omni-lang/examples/hello.omni",
            stage1_should_pass: true,
            omnc_should_pass: true,
        },
        Case {
            input_rel: "omni-lang/examples/shared_minimal.omni",
            stage1_should_pass: true,
            omnc_should_pass: true,
        },
        Case {
            input_rel: "omni-lang/examples/minimal.omni",
            stage1_should_pass: true,
            omnc_should_pass: true,
        },
        Case {
            input_rel: "omni-lang/examples/simple_test.omni",
            stage1_should_pass: true,
            omnc_should_pass: true,
        },
    ];

    for case in cases {
        let input = root.join(case.input_rel);
        assert!(input.exists(), "missing test input {}", input.display());

        let stage1_ok = run_stage1_check(&stage1_bin, &input);
        let omnc_ok = run_omnc_emit(&omnc_bin, &input);

        assert_eq!(
            stage1_ok, case.stage1_should_pass,
            "stage1 mismatch for {}: expected {}, got {}",
            input.display(), case.stage1_should_pass, stage1_ok
        );
        assert_eq!(
            omnc_ok, case.omnc_should_pass,
            "omnc mismatch for {}: expected {}, got {}",
            input.display(), case.omnc_should_pass, omnc_ok
        );
    }
}
