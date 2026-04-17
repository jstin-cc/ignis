use crate::model::UsageEvent;
use crate::parser::{self, ParseError};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

/// Platform-specific file identity used to detect rotation or replacement of a JSONL file.
///
/// On Windows: NTFS volume serial number + file index from `GetFileInformationByHandle`
///             (equivalent to Unix inode, stable without feature flags).
/// On Unix:    derived from device number + inode number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileIdentity {
    unique_id: u128,
}

/// Tracks how far into a single JSONL file we have read.
#[derive(Clone, Debug)]
pub struct FilePosition {
    pub path: PathBuf,
    pub identity: FileIdentity,
    /// Byte offset of the next unread byte (i.e. current EOF after last scan).
    pub byte_offset: u64,
}

/// Output of any scan operation.
#[derive(Debug, Default)]
pub struct ScanResult {
    pub events: Vec<UsageEvent>,
    /// One entry per successfully opened file, with the updated byte offset.
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
    for path in jsonl_files_under(root) {
        scan_file_into(&path, 0, &mut result);
    }
    result
}

/// Re-scan each file in `positions` starting from the stored byte offset.
///
/// If a file's identity has changed (rotation / replacement), the entire file
/// is re-scanned from byte 0.  Files that cannot be stat'd are recorded as
/// errors and excluded from the returned positions.
pub fn scan_delta(positions: &[FilePosition]) -> ScanResult {
    let mut result = ScanResult::default();
    for pos in positions {
        let start = match file_identity(&pos.path) {
            Ok(id) if id == pos.identity => pos.byte_offset,
            Ok(_) => 0, // file rotated — full rescan
            Err(e) => {
                result.errors.push(ScanError::Io {
                    path: pos.path.clone(),
                    source: e,
                });
                continue;
            }
        };
        scan_file_into(&pos.path, start, &mut result);
    }
    result
}

// ── Internals ─────────────────────────────────────────────────────────────────

fn jsonl_files_under(root: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some_and(|x| x == "jsonl"))
        .map(|e| e.path().to_owned())
}

fn scan_file_into(path: &Path, start_offset: u64, result: &mut ScanResult) {
    let identity = match file_identity(path) {
        Ok(id) => id,
        Err(e) => {
            result.errors.push(ScanError::Io {
                path: path.to_owned(),
                source: e,
            });
            return;
        }
    };

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
fn file_identity(path: &Path) -> io::Result<FileIdentity> {
    use std::os::windows::io::AsRawHandle;

    // BY_HANDLE_FILE_INFORMATION layout from the Windows SDK.
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

    let file = std::fs::File::open(path)?;
    let handle = file.as_raw_handle();
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: handle is valid for the lifetime of `file`; info is properly zeroed and aligned.
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
fn file_identity(path: &Path) -> io::Result<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path)?;
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

    // Minimal valid assistant JSONL lines the parser accepts.
    const LINE_A: &str = r#"{"type":"assistant","isSidechain":false,"message":{"model":"claude-sonnet-4-6","usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}},"uuid":"u1","sessionId":"s1","timestamp":"2026-04-17T09:00:00.000Z","cwd":"C:\\test"}"#;
    const LINE_B: &str = r#"{"type":"assistant","isSidechain":false,"message":{"model":"claude-sonnet-4-6","usage":{"input_tokens":20,"output_tokens":8,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}},"uuid":"u2","sessionId":"s1","timestamp":"2026-04-17T09:00:01.000Z","cwd":"C:\\test"}"#;

    /// RAII temp directory — removed on drop.
    struct TestDir(PathBuf);
    impl TestDir {
        fn new(name: &str) -> Self {
            let d = std::env::temp_dir().join(format!("winusage_scanner_test_{name}"));
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
        // fixtures/: happy-path.jsonl (2 events), error-synthetic.jsonl (0), sidechain.jsonl (1)
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
        let r = scan_all(&fixtures);
        assert_eq!(
            r.errors.len(),
            0,
            "unexpected parse/io errors: {:?}",
            r.errors
        );
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
        assert_eq!(r1.events[0].input_tokens, 10);

        // Append a second line.
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

        // Simulate rotation: delete and recreate with different content.
        std::fs::remove_file(&path).unwrap();
        std::fs::write(&path, format!("{LINE_A}\n{LINE_B}\n")).unwrap();

        // Delta with old positions pointing past EOF of the old file.
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
}
