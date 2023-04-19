use std::{ffi::OsStr, io::ErrorKind, process::Command};

use crate::Result;

/// This will check where a command exist by attempting to run it first.
///
/// - If the command does not exist, this will return `Ok(false)`;
/// - If the command exists but failed to execute, likely due to its file permission,
/// this will return the actual error;
/// - If the command exists and successfully executed, this will return `Ok(true)`;
pub fn command_exist<A: AsRef<OsStr>, V: AsRef<OsStr>>(app: A, args: &[V]) -> Result<bool> {
    if let Some(err) = Command::new(app).args(args).spawn().err() {
        if err.kind() == ErrorKind::NotFound {
            Ok(false)
        } else {
            Err(err.into())
        }
    } else {
        Ok(true)
    }
}
