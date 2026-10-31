// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors

//! A project as one file.
//!
//! A project's folder holds only the edit; the videos, sounds, pictures and
//! fonts it uses live wherever they were imported from, and the edit refers
//! to them by absolute path. That is why a project cannot simply be copied
//! to another phone, nor survive the app being uninstalled, which takes the
//! app's own folders with it. A bundle is the answer: one zip holding the
//! edit with every path rewritten to a place inside the zip, and the files
//! themselves.
//!
//! ```text
//! bundle.json   what this is: format, version, name, what is missing
//! concat.json   the edit, its media and font paths as "bundle:media/1-a.mp4"
//! preview.png   the project's picture on the launch screen, when it has one
//! media/        every file the edit uses, numbered so no two names clash
//! ```
//!
//! [`pack`] writes one; [`unpack`] reads one into a fresh project folder
//! and puts the paths back to where the files now are. Neither knows what a
//! project means beyond the two lists it keeps paths in.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// The extension a bundle is written with. Ends in ".zip" on purpose: a
/// phone's file manager and its share sheet know what a zip is, and call
/// anything else a generic file.
pub const EXTENSION: &str = "concatproj.zip";

/// The MIME type a bundle is published under.
pub const MIME: &str = "application/zip";

const KIND: &str = "concat-project-bundle";
const VERSION: u64 = 1;
const INFO: &str = "bundle.json";
const MANIFEST: &str = "concat.json";
const PREVIEW: &str = "preview.png";
const MEDIA_DIR: &str = "media/";
/// What a rewritten path starts with, to tell it from a real one.
const PREFIX: &str = "bundle:";
/// The edit's lists that hold a `path` to a file on disk.
const PATH_LISTS: [&str; 2] = ["media", "fonts"];

/// Why a bundle could not be written or read.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file could not be read or written.
    #[error("{what}: {source}")]
    Io {
        /// What was being done, with the path.
        what: String,
        /// The system's reason.
        source: io::Error,
    },
    /// The zip container itself is damaged or not a zip.
    #[error("not a readable bundle: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// A zip, but not one of ours.
    #[error("this is not a Concat project file")]
    NotABundle,
    /// Written by a newer Concat than this one.
    #[error("this project file was made by a newer Concat (format {0}); update the app to open it")]
    TooNew(u64),
    /// A part the bundle promises is not in it, or one is not what it says.
    #[error("the project file is damaged: {0}")]
    Damaged(String),
}

fn io_error(what: impl Into<String>) -> impl FnOnce(io::Error) -> Error {
    let what = what.into();
    move |source| Error::Io { what, source }
}

/// What [`pack`] put in the bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packed {
    /// How many files went in under `media/`.
    pub files: usize,
    /// Their total size in bytes.
    pub bytes: u64,
    /// Paths the edit names that are no longer on the disk. They stay in
    /// the edit as they were, and show as missing when it is opened, as
    /// they did before.
    pub missing: Vec<String>,
}

/// What [`unpack`] made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unpacked {
    /// The project's name, as the edit has it.
    pub name: String,
    /// How many files were put in the project's `media/`.
    pub files: usize,
    /// Paths the project had already lost when it was packed.
    pub missing: Vec<String>,
}

/// The project's name as the edit records it.
pub fn name_of(document: &Value) -> String {
    document
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Project")
        .to_owned()
}

/// A file name that is safe as one part of a path inside the zip and on any
/// disk: no separators, no control characters, not empty, not too long.
fn safe_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_start_matches('.');
    // Keep the extension when the name must be cut: it is what says what
    // the file is.
    let cleaned = if cleaned.len() > 100 {
        let (stem, extension) = match cleaned.rfind('.') {
            Some(dot) if cleaned.len() - dot <= 12 => cleaned.split_at(dot),
            _ => (cleaned, ""),
        };
        let stem: String = stem.chars().take(80).collect();
        format!("{stem}{extension}")
    } else {
        cleaned.to_owned()
    };
    if cleaned.is_empty() {
        "file".to_owned()
    } else {
        cleaned
    }
}

/// Writes the bundle for the project whose edit is `document` and whose
/// folder is `folder` to `out`, replacing whatever is there.
///
/// `document` is the edit as it is now - the caller saves the open project
/// first, or hands the live document - and the files it names are read
/// from where they are. `out` is written under a temporary name and moved
/// into place, so a bundle that is there is always a whole one.
pub fn pack(document: &Value, folder: &Path, out: &Path) -> Result<Packed, Error> {
    let part = out.with_extension("part");
    let result = pack_to(document, folder, &part);
    match result {
        Ok(packed) => {
            std::fs::rename(&part, out)
                .map_err(io_error(format!("could not write {}", out.display())))?;
            Ok(packed)
        }
        Err(error) => {
            let _ = std::fs::remove_file(&part);
            Err(error)
        }
    }
}

fn pack_to(document: &Value, folder: &Path, part: &Path) -> Result<Packed, Error> {
    let file =
        File::create(part).map_err(io_error(format!("could not create {}", part.display())))?;
    let mut zip = ZipWriter::new(file);
    let stored = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut document = document.clone();
    let mut entries: HashMap<String, String> = HashMap::new();
    let mut packed = Packed {
        files: 0,
        bytes: 0,
        missing: Vec::new(),
    };

    for list in PATH_LISTS {
        let Some(items) = document.get_mut(list).and_then(Value::as_array_mut) else {
            continue;
        };
        for item in items {
            let Some(path) = item.get("path").and_then(Value::as_str).map(str::to_owned) else {
                continue;
            };
            if path.starts_with(PREFIX) {
                continue;
            }
            let source = Path::new(&path);
            if !source.is_file() {
                if !packed.missing.contains(&path) {
                    packed.missing.push(path);
                }
                continue;
            }
            // The same file used twice is stored once.
            let entry = match entries.get(&path) {
                Some(entry) => entry.clone(),
                None => {
                    let base = source
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let entry = format!("{MEDIA_DIR}{}-{}", entries.len() + 1, safe_name(&base));
                    zip.start_file(&entry, stored)?;
                    let mut input = File::open(source)
                        .map_err(io_error(format!("could not read {}", source.display())))?;
                    let copied = io::copy(&mut input, &mut zip)
                        .map_err(io_error(format!("could not pack {}", source.display())))?;
                    packed.files += 1;
                    packed.bytes += copied;
                    entries.insert(path.clone(), entry.clone());
                    entry
                }
            };
            item["path"] = Value::String(format!("{PREFIX}{entry}"));
        }
    }

    let name = name_of(&document);
    let info = json!({
        "kind": KIND,
        "version": VERSION,
        "name": name,
        "app": env!("CARGO_PKG_VERSION"),
        "files": packed.files,
        "bytes": packed.bytes,
        "missing": packed.missing,
    });
    write_part(
        &mut zip,
        INFO,
        deflated,
        &serde_json::to_vec_pretty(&info).unwrap_or_default(),
    )?;
    write_part(
        &mut zip,
        MANIFEST,
        deflated,
        &serde_json::to_vec_pretty(&document).unwrap_or_default(),
    )?;
    if let Ok(picture) = std::fs::read(folder.join(PREVIEW)) {
        write_part(&mut zip, PREVIEW, stored, &picture)?;
    }

    let mut file = zip.finish()?;
    file.flush()
        .map_err(io_error(format!("could not write {}", part.display())))?;
    file.sync_all()
        .map_err(io_error(format!("could not write {}", part.display())))?;
    Ok(packed)
}

fn write_part(
    zip: &mut ZipWriter<File>,
    name: &str,
    options: SimpleFileOptions,
    bytes: &[u8],
) -> Result<(), Error> {
    zip.start_file(name, options)?;
    zip.write_all(bytes)
        .map_err(io_error(format!("could not pack {name}")))
}

/// Reads just enough of a bundle to name it - the project's name - without
/// unpacking anything. Also what tells a bundle from any other zip.
pub fn peek(bundle: &Path) -> Result<String, Error> {
    let mut archive = open_archive(bundle)?;
    Ok(read_info(&mut archive)?.0)
}

fn open_archive(bundle: &Path) -> Result<ZipArchive<File>, Error> {
    let file =
        File::open(bundle).map_err(io_error(format!("could not open {}", bundle.display())))?;
    Ok(ZipArchive::new(file)?)
}

/// The name and the missing paths, after checking the bundle is ours and
/// not newer than this.
fn read_info(archive: &mut ZipArchive<File>) -> Result<(String, Vec<String>), Error> {
    let mut text = String::new();
    match archive.by_name(INFO) {
        Ok(mut entry) => {
            // The info is a few lines; a megabyte of it is not ours.
            entry
                .by_ref()
                .take(1 << 20)
                .read_to_string(&mut text)
                .map_err(io_error("could not read the project file's header"))?;
        }
        Err(zip::result::ZipError::FileNotFound) => return Err(Error::NotABundle),
        Err(error) => return Err(error.into()),
    }
    let info: Value = serde_json::from_str(&text).map_err(|_| Error::NotABundle)?;
    if info.get("kind").and_then(Value::as_str) != Some(KIND) {
        return Err(Error::NotABundle);
    }
    let version = info.get("version").and_then(Value::as_u64).unwrap_or(0);
    if version == 0 {
        return Err(Error::NotABundle);
    }
    if version > VERSION {
        return Err(Error::TooNew(version));
    }
    let name = info
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Project")
        .to_owned();
    let missing = info
        .get("missing")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    Ok((name, missing))
}

/// Whether `name` is a place inside the bundle this crate would have
/// written: `media/` and one file name, nothing that climbs out of the
/// folder it is unpacked into.
fn is_media_entry(name: &str) -> bool {
    let Some(file) = name.strip_prefix(MEDIA_DIR) else {
        return false;
    };
    !file.is_empty()
        && file != "."
        && file != ".."
        && !file.contains(['/', '\\', ':', '\0'])
        && !file.starts_with('.')
}

/// Unpacks `bundle` into `folder`, which is created and must not already
/// hold a project, and leaves a project there that opens like any other:
/// its edit, its picture, and its files under `media/`, with the edit
/// pointing at them.
///
/// On any failure the folder is removed again, so a half-unpacked project
/// is never left behind to be opened.
pub fn unpack(bundle: &Path, folder: &Path) -> Result<Unpacked, Error> {
    if folder.exists() && std::fs::read_dir(folder).is_ok_and(|mut list| list.next().is_some()) {
        return Err(Error::Damaged(format!(
            "{} already holds files",
            folder.display()
        )));
    }
    let existed = folder.exists();
    let result = unpack_into(bundle, folder);
    if result.is_err() {
        if existed {
            // Not ours to remove, only what we put in it.
            let _ = std::fs::remove_dir_all(folder.join("media"));
            let _ = std::fs::remove_file(folder.join(MANIFEST));
            let _ = std::fs::remove_file(folder.join(PREVIEW));
        } else {
            let _ = std::fs::remove_dir_all(folder);
        }
    }
    result
}

fn unpack_into(bundle: &Path, folder: &Path) -> Result<Unpacked, Error> {
    let mut archive = open_archive(bundle)?;
    let (_, missing) = read_info(&mut archive)?;

    let mut text = String::new();
    archive
        .by_name(MANIFEST)
        .map_err(|_| Error::Damaged("it has no project in it".to_owned()))?
        .take(256 << 20)
        .read_to_string(&mut text)
        .map_err(io_error("could not read the project"))?;
    let mut document: Value = serde_json::from_str(&text)
        .map_err(|error| Error::Damaged(format!("the project is not readable: {error}")))?;

    let media = folder.join("media");
    std::fs::create_dir_all(&media)
        .map_err(io_error(format!("could not create {}", media.display())))?;

    // Every file the zip holds under media/, by the name the edit uses.
    let mut placed: HashMap<String, PathBuf> = HashMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        if entry.is_dir() || !is_media_entry(&name) {
            continue;
        }
        let target = folder.join(&name);
        let mut out = File::create(&target)
            .map_err(io_error(format!("could not create {}", target.display())))?;
        io::copy(&mut entry, &mut out).map_err(io_error(format!("could not unpack {name}")))?;
        placed.insert(name, target);
    }

    for list in PATH_LISTS {
        let Some(items) = document.get_mut(list).and_then(Value::as_array_mut) else {
            continue;
        };
        for item in items {
            let Some(path) = item.get("path").and_then(Value::as_str) else {
                continue;
            };
            let Some(entry) = path.strip_prefix(PREFIX) else {
                continue;
            };
            let target = placed.get(entry).ok_or_else(|| {
                Error::Damaged(format!(
                    "{entry} is named by the project but not in the file"
                ))
            })?;
            item["path"] = Value::String(target.to_string_lossy().into_owned());
        }
    }

    if let Ok(mut entry) = archive.by_name(PREVIEW) {
        let mut picture = Vec::new();
        if entry
            .by_ref()
            .take(64 << 20)
            .read_to_end(&mut picture)
            .is_ok()
        {
            let _ = std::fs::write(folder.join(PREVIEW), picture);
        }
    }

    let name = name_of(&document);
    std::fs::write(
        folder.join(MANIFEST),
        serde_json::to_vec_pretty(&document).unwrap_or_default(),
    )
    .map_err(io_error("could not write the project"))?;

    Ok(Unpacked {
        name,
        files: placed.len(),
        missing,
    })
}

#[cfg(test)]
mod tests;
