use crate::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

/// A wrapper to `std::fs::read_to_string`, which will try to read a file,
/// then return its content as a `String`.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path).map_err(|e| e.into())
}

fn write_<C, P>(content: C, path: P, overwrite: bool) -> Result<()>
where
    C: AsRef<[u8]>,
    P: AsRef<Path>,
{
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(overwrite)
        .create(true)
        .open(path)?;
    file.write_all(content.as_ref()).map_err(|e| e.into())
}

/// Open/Crate a file for writing and attemp to write all content into it.
///
/// This function will truncate the previous content if it has one,
/// if you want to extend the file content, use [`append_to_file`] instead.
pub fn write_to_file<C, P>(content: C, path: P) -> Result<()>
where
    C: AsRef<[u8]>,
    P: AsRef<Path>,
{
    write_(content, path, true)
}

/// Open/Crate a file for writing and attemp to append all content to it.
pub fn append_to_file<C, P>(content: C, path: P) -> Result<()>
where
    C: AsRef<[u8]>,
    P: AsRef<Path>,
{
    write_(content, path, false)
}
