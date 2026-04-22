use crate::model::UsageEvent;
use crate::parser::{self, ParseError};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

/// Platform-specific file identity used to detect rotation or replacement of a JSONL file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileIdentity {
    unique_id: u128,
}

/// Tracks how far into a single JSONL file we have read.
#[derive(Clone, Debug)]
pub struct FilePosition {
    pub path: PathBuf,
    pub identity: FileIdentity,
    pub byte_offset: u64,
}

/// Output of any scan operation.
#[derive(Debug, Default)]
pub struct ScanResult {
    pub events: Vec<UsageEvent>,
    pub positions: Vec<FilePosition>,
    pub errors: Vec<ScanError>,
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("io error for '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("parse error in '{path}' at line {line}: {source}")]
    Parse {
        path: PathBuf,
        line: usize,
        source: ParseError,
    },
}

/// Scan every `*.jsonl` file found (recursively) under `root` from byte 0.
pub fn scan_all(root: &Path) -> ScanResult {
    let mut result = ScanResult::default();
    let mut paths = Vec::new();
    walk_jsonl(root, &mut result, |p| paths.push(p));
    for path in paths {
        scan_file_into(&path, 0, &mut result);
    }
    result
}

/// Re-scan each file in `positions` starting from the stored byte offset.
///
/// If a file's identity has changed (rotation / replacement), the entire file
/// is re-scanned from byte 0. Files that cannot be opened are recorded as
/// errors and excluded from the returned positions.
///
/// The file is opened once and identity derived from the open handle to
/// prevent a TOCTOU race between the identity check and the read.
pub fn scan_delta(positions: &[FilePosition]) -> ScanResult {
    let mut result = ScanResult::default();
    for pos in positions {
        let file = match File::open(&pos.path) {
            Ok(f) => f,
            Err(e) => {
                result.errors.push(ScanError::Io {
                    path: pos.path.clone(),
                    source: e,
                });
                continue;
            }
        };
        let current_id = match file_identity_handle(&file) {
            Ok(id) => id,
            Err(e) => {
                result.errors.push(ScanError::Io {
                    path: pos.path.clone(),
                    source: e,
                });
                continue;
            }
        };
        let start = if current_id == pos.identity {
            pos.byte_offset
        } else {
            0
        };
        scan_open_file(file, &pos.path, current_id, start, &mut result);
    }
    result
}

/// Delta scan for known files + full scan for newly discovered files.
///
/// This is the preferred refresh operation for long-running watchers: it reads
/// only new bytes for known files while still picking up any `.jsonl` files
/// that appeared since the last scan.
pub fn scan_incremental(root: &Path, positions: &[FilePosition]) -> ScanResult {
    let mut result = ScanResult::default();

    // Delta for known files.
    for pos in positions {
        let file = match File::open(&pos.path) {
            Ok(f) => f,
            Err(e) => {
                result.errors.push(ScanError::Io {
                    path: pos.path.clone(),
                    source: e,
                });
                continue;
            }
        };
        let current_id = match file_identity_handle(&file) {
            Ok(id) => id,
            Err(e) => {
                result.errors.push(ScanError::Io {
                    path: pos.path.clone(),
                    source: e,
                });
                continue;
            }
        };
        let start = if current_id == pos.identity {
            pos.byte_offset
        } else {
            0
        };
        scan_open_file(file, &pos.path, current_id, start, &mut result);
    }

    // Discover new files not yet in positions.
    let known: HashSet<&Path> = positions.iter().map(|p| p.path.as_path()).collect();
    let mut new_paths = Vec::new();
    walk_jsonl(root, &mut result, |path| {
        if !known.contains(path.as_path()) {
            new_paths.push(path);
        }
    });
    for path in new_paths {
        scan_file_into(&path, 0, &mut result);
    }

    result
}

// ── Internals ─────────────────────────────────────────────────────────────────

/// Walk `root`, add any WalkDir errors to `result.errors`, and call `f` for
/// each `.jsonl` file found.
fn walk_jsonl<F: FnMut(PathBuf)>(root: &Path, result: &mut ScanResult, mut f: F) {
    for entry in WalkDir::new(root).follow_links(false) {
        match entry {
            Err(e) => {
                let path = e.path().map(|p| p.to_owned()).unwrap_or_default();
                result.errors.push(ScanError::Io {
                    path,
                    source: e
                        .into_io_error()
                        .unwrap_or_else(|| io::Error::other("walk error")),
                });
            }
            Ok(ent)
                if ent.file_type().is_file()
                    && ent.path().extension().is_some_and(|x| x == "jsonl") =>
            {
                f(ent.path().to_owned());
            }
            Ok(_) => {}
        }
    }
}

/// Open `path`, derive its identity from the handle, then scan from `start_offset`.
fn scan_file_into(path: &Path, start_offset: u64, result: &mut ScanResult) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            result.errors.push(ScanError::Io {
                path: path.to_owned(),
                source: e,
            });
            return;
        }
    };
    let identity = match file_identity_handle(&file) {
        Ok(id) => id,
        Err(e) => {
            result.errors.push(ScanError::Io {
                path: path.to_owned(),
                source: e,
            });
            return;
        }
    };
    scan_open_file(file, path, identity, start_offset, result);
}

/// Read lines from an already-open file starting at `start_offset`.
fn scan_open_file(
    file: File,
    path: &Path,
    identity: FileIdentity,
    start_offset: u64,
    result: &mut ScanResult,
) {
    let mut reader = BufReader::new(file);
    if start_offset > 0 {
        if let Err(e) = reader.seek(SeekFrom::Start(start_offset)) {
            result.errors.push(ScanError::Io {
                path: path.to_owned(),
                source: e,
            });
            return;
        }
    }

    let mut byte_offset = start_offset;
    let mut line_num = 0usize;
    let mut buf = String::new();

    loop {
        buf.clear();
        match reader.read_line(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                line_num += 1;
                let line = buf.trim_end_matches(['\r', '\n']);
                if !line.is_empty() {
                    match parser::parse_line(line) {
                        Ok(Some(ev)) => result.events.push(ev),
                        Ok(None) => {}
                        Err(e) => result.errors.push(ScanError::Parse {
                            path: path.to_owned(),
                            line: line_num,
                            source: e,
                        }),
                    }
                }
                byte_offset += n as u64;
            }
            Err(e) => {
                result.errors.push(ScanError::Io {
                    path: path.to_owned(),
                    source: e,
                });
                break;
            }
        }
    }

    result.positions.push(FilePosition {
        path: path.to_owned(),
        identity,
        byte_offset,
    });
}

#[cfg(windows)]
fn file_identity_handle(file: &File) -> io::Result<FileIdentity> {
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct BY_HANDLE_FILE_INFORMATION {
        dwFileAttributes: u32,
        ftCreationTime: [u32; 2],
        ftLastAccessTime: [u32; 2],
        ftLastWriteTime: [u32; 2],
        dwVolumeSerialNumber: u32,
        nFileSizeHigh: u32,
        nFileSizeLow: u32,
        nNumberOfLinks: u32,
        nFileIndexHigh: u32,
        nFileIndexLow: u32,
    }

    extern "system" {
        fn GetFileInformationByHandle(
            hFile: *mut std::ffi::c_void,
            lpFileInformation: *mut BY_HANDLE_FILE_INFORMATION,
        ) -> i32;
    }

    let handle = file.as_raw_handle();
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: handle is valid for the lifetime of `file`; info is zeroed and aligned.
    if unsafe { GetFileInformationByHandle(handle, &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let vol = info.dwVolumeSerialNumber as u128;
    let idx = ((info.nFileIndexHigh as u128) << 32) | (info.nFileIndexLow as u128);
    Ok(FileIdentity {
        unique_id: (vol << 64) | idx,
    })
}

#[cfg(not(windows))]
fn file_identity_handle(file: &File) -> io::Result<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let meta = file.metadata()?;
    Ok(FileIdentity {
        unique_id: ((meta.dev() as u128) << 64) | (meta.ino() as u128),
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::Write;

    const LINE_A: &str = r#"{"type":"assistant","isSidechain":false,"message":{"model":"claude-sonnet-4-6","usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}},"uuid":"u1","sessionId":"s1","timestamp":"2026-04-17T09:00:00.000Z","cwd":"C:\\test"}"#;
    const LINE_B: &str = r#"{"type":"assistant","isSidechain":false,"message":{"model":"claude-sonnet-4-6","usage":{"input_tokens":20,"output_tokens":8,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}},"uuid":"u2","sessionId":"s1","timestamp":"2026-04-17T09:00:01.000Z","cwd":"C:\\test"}"#;

    struct TestDir(PathBuf);
    impl TestDir {
        fn new(name: &str) -> Self {
            let d = std::env::temp_dir().join(format!("ignis_scanner_test_{name}"));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).unwrap();
            Self(d)
        }
        fn path(&self) -> &Path {
            &self.0
        }
        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn scan_all_finds_fixture_events() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
        let r = scan_all(&fixtures);
        assert_eq!(r.errors.len(), 0, "unexpected errors: {:?}", r.errors);
        assert_eq!(r.events.len(), 3);
        assert_eq!(r.positions.len(), 3);
    }

    #[test]
    fn scan_delta_reads_only_new_bytes() {
        let dir = TestDir::new("delta");
        let path = dir.file("session.jsonl");
        std::fs::write(&path, format!("{LINE_A}\n")).unwrap();

        let r1 = scan_all(dir.path());
        assert_eq!(r1.events.len(), 1);

        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        writeln!(f, "{LINE_B}").unwrap();
        drop(f);

        let r2 = scan_delta(&r1.positions);
        assert_eq!(r2.errors.len(), 0);
        assert_eq!(r2.events.len(), 1, "only the newly appended line");
        assert_eq!(r2.events[0].input_tokens, 20);
    }

    #[test]
    fn scan_delta_detects_rotation_and_rescans_from_zero() {
        let dir = TestDir::new("rotation");
        let path = dir.file("session.jsonl");
        std::fs::write(&path, format!("{LINE_A}\n")).unwrap();
        let r1 = scan_all(dir.path());
        assert_eq!(r1.events.len(), 1);

        let tmp = dir.file("session.jsonl.tmp");
        std::fs::write(&tmp, format!("{LINE_A}\n{LINE_B}\n")).unwrap();
        std::fs::rename(&tmp, &path).unwrap();

        let r2 = scan_delta(&r1.positions);
        assert_eq!(r2.errors.len(), 0);
        assert_eq!(r2.events.len(), 2, "full rescan after rotation");
    }

    #[test]
    fn scan_delta_missing_file_emits_error() {
        let pos = FilePosition {
            path: PathBuf::from("C:\\nonexistent\\missing.jsonl"),
            identity: FileIdentity { unique_id: 0 },
            byte_offset: 0,
        };
        let r = scan_delta(&[pos]);
        assert_eq!(r.events.len(), 0);
        assert_eq!(r.errors.len(), 1);
        assert!(matches!(r.errors[0], ScanError::Io { .. }));
    }

    #[test]
    fn scan_all_ignores_non_jsonl_files() {
        let dir = TestDir::new("non_jsonl");
        std::fs::write(dir.file("notes.txt"), "not jsonl").unwrap();
        std::fs::write(dir.file("data.json"), "{}").unwrap();
        let r = scan_all(dir.path());
        assert_eq!(r.events.len(), 0);
        assert_eq!(r.positions.len(), 0);
        assert_eq!(r.errors.len(), 0);
    }

    #[test]
    fn scan_incremental_picks_up_new_files() {
        let dir = TestDir::new("incremental");
        let path_a = dir.file("a.jsonl");
        std::fs::write(&path_a, format!("{LINE_A}\n")).unwrap();

        let r1 = scan_all(dir.path());
        assert_eq!(r1.events.len(), 1);

        // New file appears.
        let path_b = dir.file("b.jsonl");
        std::fs::write(&path_b, format!("{LINE_B}\n")).unwrap();

        let r2 = scan_incremental(dir.path(), &r1.positions);
        assert_eq!(r2.errors.len(), 0);
        // 0 new events from a.jsonl (unchanged) + 1 from b.jsonl
        assert_eq!(r2.events.len(), 1);
        assert_eq!(r2.events[0].input_tokens, 20);
        // positions: both files
        assert_eq!(r2.positions.len(), 2);
    }

    #[test]
    fn scan_incremental_delta_and_new_combined() {
        let dir = TestDir::new("incremental_combined");
        let path_a = dir.file("a.jsonl");
        std::fs::write(&path_a, format!("{LINE_A}\n")).unwrap();

        let r1 = scan_all(dir.path());
        assert_eq!(r1.events.len(), 1);

        // Append to existing + add new file.
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path_a)
            .unwrap();
        writeln!(f, "{LINE_B}").unwrap();
        drop(f);

        let path_b = dir.file("b.jsonl");
        std::fs::write(&path_b, format!("{LINE_A}\n")).unwrap();

        let r2 = scan_incremental(dir.path(), &r1.positions);
        assert_eq!(r2.errors.len(), 0);
        // 1 new from a (appended) + 1 from b (new file)
        assert_eq!(r2.events.len(), 2);
    }
}
