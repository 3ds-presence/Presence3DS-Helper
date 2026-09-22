use std::fmt;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read, Write};

use super::sd_card::SdCardInfo;
pub const SDMC_PREFIX: &str = "sdmc:";
pub const MAX_PATH_LEN: usize = 1024;
pub const MAX_COMPONENT_LEN: usize = 255;
pub const DEFAULT_CHUNK_SIZE: usize = 32 * 1024;
pub const TEMP_SUFFIX: &str = ".part";

#[derive(Debug)]
pub enum SdCardError {
    InvalidPath {
        path: String,
        reason: &'static str,
    },
    NotEnoughSpace {
        path: String,
        needed: usize,
        free: usize,
    },
    InvalidChunk {
        size: usize,
    },
    ChunkTooLarge {
        max: usize,
        got: usize,
    },
    SizeMismatch {
        path: String,
        expected: u64,
        actual: u64,
    },
    EmptyStagedFile {
        path: String,
    },
    Io {
        path: String,
        source: io::Error,
    },
}

impl fmt::Display for SdCardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { path, reason } => {
                write!(f, "invalid SD card path {path:?}: {reason}")
            }
            Self::NotEnoughSpace { path, needed, free } => write!(
                f,
                "not enough space on the SD card for {path:?}: {needed} bytes needed, {free} bytes free"
            ),
            Self::InvalidChunk { size } => {
                write!(f, "invalid chunk size: {size} bytes")
            }
            Self::ChunkTooLarge { max, got } => write!(
                f,
                "chunk of {got} bytes is bigger than the maximum chunk size of {max} bytes"
            ),
            Self::SizeMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "size mismatch after writing {path:?}: expected {expected} bytes, found {actual}"
            ),
            Self::EmptyStagedFile { path } => {
                write!(f, "refusing to install the empty file {path:?}")
            }
            Self::Io { path, source } => {
                write!(f, "SD card I/O error on {path:?}: {source}")
            }
        }
    }
}

impl std::error::Error for SdCardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn io_error(path: &str, source: io::Error) -> SdCardError {
    SdCardError::Io {
        path: path.to_owned(),
        source,
    }
}

pub fn sd_path(path: &str) -> Result<String, SdCardError> {
    let invalid = |reason: &'static str| SdCardError::InvalidPath {
        path: path.to_owned(),
        reason,
    };

    let without_device = path.strip_prefix(SDMC_PREFIX).unwrap_or(path);
    if without_device.trim().is_empty() {
        return Err(invalid("path is empty"));
    }

    let mut components = Vec::new();
    for component in without_device.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            return Err(invalid("parent directory traversal is not allowed"));
        }
        if component.len() > MAX_COMPONENT_LEN {
            return Err(invalid("path component is longer than 255 bytes"));
        }
        if component.ends_with('.') || component.ends_with(' ') {
            return Err(invalid("path component ends with a dot or a space"));
        }
        for character in component.chars() {
            match character {
                '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*' => {
                    return Err(invalid(
                        "path component contains a character forbidden by FAT",
                    ));
                }
                character if character.is_control() => {
                    return Err(invalid("path component contains a control character"));
                }
                _ => {}
            }
        }
        components.push(component);
    }

    if components.is_empty() {
        return Err(invalid("path does not contain any file or directory name"));
    }

    let normalized = format!("{SDMC_PREFIX}/{}", components.join("/"));
    if normalized.len() > MAX_PATH_LEN {
        return Err(invalid("path is longer than 1024 bytes"));
    }

    Ok(normalized)
}

fn storage_chunk_limit() -> Option<usize> {
    SdCardInfo::query()
        .map(|info| info.cluster_size)
        .and_then(|cluster_size| usize::try_from(cluster_size).ok())
        .filter(|cluster_size| *cluster_size > 0)
}

pub struct SdWriter {
    file: Option<File>,
    temp_path: String,
    final_path: String,
    written: u64,
    max_chunk: usize,
}

impl SdWriter {
    pub fn create(path: &str) -> Result<Self, SdCardError> {
        Self::create_with_chunk(path, storage_chunk_limit().unwrap_or(DEFAULT_CHUNK_SIZE))
    }

    pub fn create_with_chunk(path: &str, chunk_size: usize) -> Result<Self, SdCardError> {
        if chunk_size == 0 || chunk_size > u32::MAX as usize {
            return Err(SdCardError::InvalidChunk { size: chunk_size });
        }

        if let Some(cluster_size) = storage_chunk_limit()
            && chunk_size > cluster_size
        {
            return Err(SdCardError::ChunkTooLarge {
                max: cluster_size,
                got: chunk_size,
            });
        }

        let final_path = sd_path(path)?;
        let temp_path = format!("{final_path}{TEMP_SUFFIX}");
        if temp_path.len() > MAX_PATH_LEN {
            return Err(SdCardError::InvalidPath {
                path: path.to_owned(),
                reason: "path is too long to create the temporary file",
            });
        }

        let file = File::create(&temp_path).map_err(|error| io_error(&temp_path, error))?;

        Ok(Self {
            file: Some(file),
            temp_path,
            final_path,
            written: 0,
            max_chunk: chunk_size,
        })
    }

    #[must_use]
    pub fn final_path(&self) -> &str {
        &self.final_path
    }

    #[must_use]
    pub const fn max_chunk(&self) -> usize {
        self.max_chunk
    }

    #[must_use]
    pub const fn written(&self) -> u64 {
        self.written
    }

    pub fn ensure_capacity(&self, needed: usize) -> Result<(), SdCardError> {
        if let Some(info) = SdCardInfo::query() {
            let free = info.free_size();
            if free < needed {
                return Err(SdCardError::NotEnoughSpace {
                    path: self.final_path.clone(),
                    needed,
                    free,
                });
            }
        }

        Ok(())
    }

    pub fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SdCardError> {
        if chunk.is_empty() {
            return Ok(());
        }

        if chunk.len() > self.max_chunk {
            return Err(SdCardError::ChunkTooLarge {
                max: self.max_chunk,
                got: chunk.len(),
            });
        }

        let Some(file) = self.file.as_mut() else {
            return Err(io_error(
                &self.temp_path,
                io::Error::other("the SD writer is already closed"),
            ));
        };

        file.write_all(chunk)
            .map_err(|error| io_error(&self.temp_path, error))?;
        self.written += chunk.len() as u64;

        Ok(())
    }

    pub fn flush_to_disk(&mut self) -> Result<(), SdCardError> {
        let Some(file) = self.file.as_mut() else {
            return Err(io_error(
                &self.temp_path,
                io::Error::other("the SD writer is already closed"),
            ));
        };

        file.sync_all()
            .map_err(|error| io_error(&self.temp_path, error))
    }

    pub fn commit(mut self) -> Result<(), SdCardError> {
        if self.written == 0 {
            return Err(SdCardError::EmptyStagedFile {
                path: self.final_path.clone(),
            });
        }

        let Some(file) = self.file.take() else {
            return Err(io_error(
                &self.temp_path,
                io::Error::other("the SD writer is already closed"),
            ));
        };

        file.sync_all()
            .map_err(|error| io_error(&self.temp_path, error))?;
        drop(file);

        match fs::remove_file(&self.final_path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(io_error(&self.final_path, error)),
        }

        fs::rename(&self.temp_path, &self.final_path)
            .map_err(|error| io_error(&self.temp_path, error))?;

        let actual = fs::metadata(&self.final_path)
            .map(|metadata| metadata.len())
            .map_err(|error| io_error(&self.final_path, error))?;
        if actual != self.written {
            return Err(SdCardError::SizeMismatch {
                path: self.final_path.clone(),
                expected: self.written,
                actual,
            });
        }

        Ok(())
    }

    pub fn abort(mut self) -> Result<(), SdCardError> {
        drop(self.file.take());

        match fs::remove_file(&self.temp_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io_error(&self.temp_path, error)),
        }
    }
}

impl Write for SdWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let length = buffer.len().min(self.max_chunk);
        self.write_chunk(&buffer[..length])
            .map_err(io::Error::other)?;

        Ok(length)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_to_disk().map_err(io::Error::other)
    }
}

impl Drop for SdWriter {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            drop(file);
            let _ = fs::remove_file(&self.temp_path);
        }
    }
}

pub fn write_file(path: &str, data: &[u8]) -> Result<(), SdCardError> {
    let mut writer = SdWriter::create(path)?;
    let chunk_size = writer.max_chunk();

    writer.ensure_capacity(data.len())?;

    for chunk in data.chunks(chunk_size) {
        writer.write_chunk(chunk)?;
    }

    writer.commit()
}

pub fn write_stream<R: Read>(path: &str, reader: &mut R) -> Result<u64, SdCardError> {
    let mut writer = SdWriter::create(path)?;
    let chunk_size = writer.max_chunk();
    let mut buffer = vec![0_u8; chunk_size];

    writer.ensure_capacity(chunk_size)?;

    loop {
        let read = match reader.read(&mut buffer) {
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(io_error(writer.final_path(), error)),
        };

        if read == 0 {
            break;
        }

        writer.write_chunk(&buffer[..read])?;
    }

    let written = writer.written();
    writer.commit()?;

    Ok(written)
}

pub fn move_file(from: &str, to: &str) -> Result<(), SdCardError> {
    let source = sd_path(from)?;
    let destination = sd_path(to)?;

    if source == destination {
        return Ok(());
    }

    match fs::remove_file(&destination) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(&destination, error)),
    }

    fs::rename(&source, &destination).map_err(|error| io_error(&source, error))
}

pub fn delete_file(path: &str) -> Result<(), SdCardError> {
    let path = sd_path(path)?;

    fs::remove_file(&path).map_err(|error| io_error(&path, error))
}

pub fn file_size(path: &str) -> Result<u64, SdCardError> {
    let path = sd_path(path)?;

    fs::metadata(&path)
        .map(|metadata| metadata.len())
        .map_err(|error| io_error(&path, error))
}

#[must_use]
pub fn file_exists(path: &str) -> bool {
    sd_path(path).is_ok_and(|path| fs::metadata(path).is_ok_and(|metadata| metadata.is_file()))
}

pub fn ensure_dir(path: &str) -> Result<(), SdCardError> {
    let path = sd_path(path)?;

    fs::create_dir_all(&path).map_err(|error| io_error(&path, error))
}
