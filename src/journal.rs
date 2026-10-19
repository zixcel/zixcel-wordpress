use crate::{Error, Result};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::{
    fs::OpenOptions,
    io::{Seek, SeekFrom, Write},
    path::Path,
};
pub(crate) struct Journal(std::fs::File);
impl Journal {
    pub(crate) fn begin(directory: &Path, digest: &str) -> Result<Self> {
        let meta = std::fs::symlink_metadata(directory).map_err(|_| Error::LocalState)?;
        let own = std::fs::metadata("/proc/self")
            .map_err(|_| Error::LocalState)?
            .uid();
        if !meta.is_dir()
            || meta.file_type().is_symlink()
            || meta.uid() != own
            || meta.permissions().mode() & 0o077 != 0
            || digest.len() != 64
            || !digest.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Error::LocalState);
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(directory.join(format!("{digest}.json")))
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    Error::Replay
                } else {
                    Error::LocalState
                }
            })?;
        file.write_all(b"{\"state\":\"write-starting; do not replay\"}\n")
            .and_then(|_| file.sync_all())
            .map_err(|_| Error::LocalState)?;
        // Persist the directory entry as well as the write-ahead file.
        std::fs::File::open(directory)
            .and_then(|d| d.sync_all())
            .map_err(|_| Error::LocalState)?;
        Ok(Self(file))
    }
    pub(crate) fn accepted(&mut self, origin: &str, id: u64, digest: &str) -> Result<()> {
        self.finish(
            &serde_json::json!({"state":"accepted; do not replay","origin":origin,
            "id":id,"plan_sha256":digest,"read_back":false}),
        )
        .map_err(|_| Error::UnknownResult)
    }
    pub(crate) fn finish(&mut self, receipt: &impl serde::Serialize) -> Result<()> {
        let bytes = serde_json::to_vec(receipt).map_err(|_| Error::LocalState)?;
        self.0
            .set_len(0)
            .and_then(|_| self.0.seek(SeekFrom::Start(0)))
            .and_then(|_| self.0.write_all(&bytes))
            .and_then(|_| self.0.sync_all())
            .map_err(|_| Error::LocalState)
    }
}
