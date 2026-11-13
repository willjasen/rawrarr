use std::{
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read},
    path::Path,
};
use std::os::unix::fs::OpenOptionsExt;

pub fn ensure_directory(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected a non-symlink directory",
        ));
    }
    Ok(())
}

pub fn open_regular_file(path: &Path) -> io::Result<(File, Metadata)> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected a regular file",
        ));
    }
    Ok((file, metadata))
}

pub fn read_regular_file(path: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
    let (file, metadata) = open_regular_file(path)?;
    if metadata.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file exceeds the permitted size",
        ));
    }
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file exceeds the permitted size",
        ));
    }
    Ok(bytes)
}

pub fn copy_open_file(mut source: File, destination: &Path) -> io::Result<()> {
    let mut target = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)?;
    let result = io::copy(&mut source, &mut target).and_then(|_| target.sync_all());
    if result.is_err() {
        drop(target);
        let _ = fs::remove_file(destination);
    }
    result.map(|_| ())
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_safe_fs.rs"]
mod tests;
