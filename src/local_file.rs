use std::{io::Read, os::unix::fs::MetadataExt, path::Path};
use zixcel_wordpress::{Error, Result};
pub(crate) fn read(path: &str) -> Result<Vec<u8>> {
    let path = Path::new(path);
    let before = std::fs::symlink_metadata(path).map_err(|_| Error::LocalState)?;
    let own = std::fs::metadata("/proc/self")
        .map_err(|_| Error::LocalState)?
        .uid();
    if !before.is_file()
        || before.file_type().is_symlink()
        || before.uid() != own
        || before.len() > 8 * 1024 * 1024
    {
        return Err(Error::InvalidInput);
    }
    let f = std::fs::File::open(path).map_err(|_| Error::LocalState)?;
    let opened = f.metadata().map_err(|_| Error::LocalState)?;
    if opened.ino() != before.ino() || opened.dev() != before.dev() {
        return Err(Error::LocalState);
    }
    let mut bytes = Vec::new();
    f.take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::LocalState)?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(Error::ResponseLimit);
    }
    Ok(bytes)
}

pub(crate) fn plan(path: &str) -> Result<Vec<u8>> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::symlink_metadata(path).map_err(|_| Error::LocalState)?;
    if meta.permissions().mode() & 0o077 != 0 || meta.len() > 1048576 {
        return Err(Error::LocalState);
    }
    read(path)
}
