use lazy_static::lazy_static;
use std::path::PathBuf;
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

        let manifest_dir = PathBuf::from(format!("{}", env!("CARGO_MANIFEST_DIR")));
        let output_dir = manifest_dir.join("target").join("output");
        fs::create_dir_all(&output_dir).unwrap();

        let test_dir = manifest_dir.join("tests");

        TestCfg {
            bin_path,
            output_dir,
            test_dir,
        }
    };
}

#[derive(Debug, Clone)]
pub struct TestCfg {
    /// Path to the actual program's binary, not the unit test binary.
    pub bin_path: PathBuf,
    pub output_dir: PathBuf,
    pub test_dir: PathBuf,
}

pub fn setup<F>(f: F)
where
    F: FnOnce(&TestCfg) -> () + panic::UnwindSafe,
{
    panic::catch_unwind(|| {
        f(&TEST_CFG);
    })
    .unwrap();
}
