use std::{
    ffi::OsStr,
    io::ErrorKind,
    process::{Child, Command, Output},
};

use crate::Result;

/// Execute a command with args and environment variables,
/// and return a handle to the child process.
pub fn execute<A, K, V, I>(app: A, args: &[V], vars: I) -> Result<Child>
where
    A: AsRef<OsStr>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
    I: IntoIterator<Item = (K, V)>,
{
    Ok(Command::new(app).args(args).envs(vars).spawn()?)
}

/// Similar to [`execute`], but will wait for the child process to finish and
/// return its output.
pub fn execute_for_output<A, K, V, I>(app: A, args: &[V], vars: I) -> Result<Output>
where
    A: AsRef<OsStr>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
    I: IntoIterator<Item = (K, V)>,
{
    Ok(Command::new(app).args(args).envs(vars).output()?)
}

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
