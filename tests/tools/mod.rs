mod lints;
mod miri;

use std::path::{Path, PathBuf};
use std::sync::Once;

static mut MOCK_CRATE: Option<PathBuf> = None;
static MOCK_CRATE_ONCE: Once = Once::new();

pub(super) fn mock_dir() -> &'static Path {
    MOCK_CRATE_ONCE.call_once(|| unsafe {
        MOCK_CRATE = Some(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("mock"),
        );
    });
    unsafe { MOCK_CRATE.as_deref().unwrap() }
}
