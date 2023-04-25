use lazy_static::lazy_static;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs, panic};

lazy_static! {
    static ref TEST_CFG: TestCfg = {
        #[cfg(windows)]
        let ext = ".exe";
        #[cfg(not(windows))]
        let ext = "";

        let cur_exe = env::current_exe().unwrap();
        let debug_dir = cur_exe.parent().and_then(|dep| dep.parent()).unwrap();
        let bin_path = debug_dir.join(format!("{}{ext}", env!("CARGO_PKG_NAME")));

        let output_dir = PathBuf::from(format!("{}", env!("CARGO_MANIFEST_DIR")))
            .join("target")
            .join("output");
        fs::create_dir_all(&output_dir).unwrap();

        TestCfg {
            bin_path,
            output_dir,
        }
    };
}

#[derive(Debug, Clone)]
struct TestCfg {
    /// Path to the actual program's binary, not the unit test binary.
    bin_path: PathBuf,
    output_dir: PathBuf,
}

fn setup<F>(f: F)
where
    F: FnOnce(&TestCfg) -> () + panic::UnwindSafe,
{
    panic::catch_unwind(|| {
        f(&TEST_CFG);
    })
    .unwrap();
}

#[test]
fn print_version() {
    setup(|cfg| {
        let res = Command::new(&cfg.bin_path).arg("-V").output().unwrap();
        let output_str = String::from_utf8(res.stdout).unwrap();
        assert_eq!(
            output_str,
            format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        );
    });
}

#[test]
fn run_with_rules() {
    setup(|cfg| {
        let op = Command::new(&cfg.bin_path)
            .args([
                "--rule-file",
                &format!(
                    "{}/tests/data/default_rules.json",
                    env!("CARGO_MANIFEST_DIR")
                ),
                "--output",
                cfg.output_dir.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(op.stdout), Ok(String::new()));
    });
}
