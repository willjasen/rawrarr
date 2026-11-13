use super::*;
use std::{fs, path::PathBuf};

fn temporary_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rawrarr-safe-fs-{name}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ))
}

#[test]
fn bounded_read_accepts_regular_files_and_rejects_oversized_files() {
    let path = temporary_path("regular");
    fs::write(&path, b"ciphertext").unwrap();
    assert_eq!(read_regular_file(&path, 10).unwrap(), b"ciphertext");
    assert_eq!(
        read_regular_file(&path, 9).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
    fs::remove_file(path).unwrap();
}

#[cfg(unix)]
#[test]
fn regular_file_open_rejects_symlinks_and_directories() {
    use std::os::unix::fs::symlink;

    let target = temporary_path("target");
    let link = temporary_path("link");
    let linked_directory = temporary_path("linked-directory");
    let directory_target = temporary_path("directory-target");
    let directory = temporary_path("directory");
    fs::write(&target, b"ciphertext").unwrap();
    symlink(&target, &link).unwrap();
    fs::create_dir(&directory_target).unwrap();
    symlink(&directory_target, &linked_directory).unwrap();
    fs::create_dir(&directory).unwrap();

    assert!(open_regular_file(&link).is_err());
    assert!(ensure_directory(&linked_directory).is_err());
    assert_eq!(
        open_regular_file(&directory).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );

    fs::remove_file(link).unwrap();
    fs::remove_file(target).unwrap();
    fs::remove_file(linked_directory).unwrap();
    fs::remove_dir(directory_target).unwrap();
    fs::remove_dir(directory).unwrap();
}
