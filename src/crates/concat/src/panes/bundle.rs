// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors

//! A project as one file: the verbs that pack one and open one.
//!
//! A project's folder holds the edit; its videos live wherever they were
//! imported from, and on a phone that is the app's own folder, which an
//! uninstall takes with it. Exporting a project file writes the edit and
//! every file it uses into one zip - on a phone into Download/Concat, where
//! the file manager shows it and a reinstall leaves it - and opening one
//! unpacks it into a new project folder. The format is `concat-bundle`'s.

use std::path::{Path, PathBuf};

use concat_host::projects;

use crate::host::{after_dialog, on_ui, pick_files_then, spawn};
use crate::i18n::{t, tf};
use crate::panes::Msg;
use crate::panes::start::StartMsg;
use crate::platform;
use crate::studio::{Studio, home_folder};

/// What a project file is picked with, and may be dropped as.
pub const EXTENSIONS: &[&str] = &["zip", "concatproj"];

/// Whether a dropped or picked path looks like a project file: a zip that
/// is ours is told from any other zip only once it is opened.
pub fn looks_like_project_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                EXTENSIONS
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(extension))
            })
}

/// Packs the project in `folder` - whose edit is `document`, as it stands
/// now - into a project file, and says where it went.
///
/// A desktop asks where to put it; a phone has no such dialog, and writes
/// to a place of its own first, then has the phone move the file to where
/// its file manager shows it.
pub fn export(studio: &mut Studio, folder: String, document: serde_json::Value) {
    let name = concat_bundle::name_of(&document);
    let file = format!(
        "{}.{}",
        projects::folder_name(&name),
        concat_bundle::EXTENSION
    );
    let beside = Path::new(&folder)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(home_folder("Documents")));

    if cfg!(any(target_os = "android", target_os = "ios")) {
        // A hidden folder beside the projects: the file only waits here for
        // the phone to take it.
        let out = beside.join(".project-files").join(file);
        pack(studio, folder, document, name, out, true);
        return;
    }

    let title = t("project.exportFile");
    let family = t("project.fileFamily");
    after_dialog(
        move || {
            let chosen = platform::save_file(&title, &beside, &file, (family.as_str(), &["zip"]))?;
            Some(
                if chosen
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
                {
                    chosen
                } else {
                    let mut name = chosen.into_os_string();
                    name.push(".zip");
                    PathBuf::from(name)
                },
            )
        },
        move |studio, _, _, out| pack(studio, folder, document, name, out, false),
    );
}

/// Packs on a worker and reports. `publish` hands the finished file to the
/// phone, which keeps it where the person can find it.
fn pack(
    studio: &mut Studio,
    folder: String,
    document: serde_json::Value,
    name: String,
    out: PathBuf,
    publish: bool,
) {
    studio.notify(&tf("project.packing", &[&name]), false);
    let target = out.clone();
    spawn(
        move || {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("{}: {error}", parent.display()))?;
            }
            concat_bundle::pack(&document, Path::new(&folder), &target).map_err(|e| e.to_string())
        },
        move |studio, _, _, result| match result {
            Err(error) => {
                log::warn!("project file: could not pack {}: {error}", out.display());
                studio.notify(&tf("project.couldNotExport", &[&error]), true);
            }
            Ok(packed) => {
                let missing = packed.missing.len();
                let say = move |studio: &mut Studio, place: &str| {
                    let said = if missing == 0 {
                        tf("project.exported", &[&place])
                    } else {
                        tf("project.exportedMissing", &[&place, &missing])
                    };
                    studio.notify(&said, false);
                };
                if !publish {
                    say(studio, &out.display().to_string());
                    return;
                }
                let waiting = out.clone();
                let handed =
                    platform::publish_document(out.clone(), concat_bundle::MIME, move |result| {
                        // Published or not, the copy waiting here is of no
                        // use to anyone but the phone's file manager.
                        let published = result.is_ok();
                        if published {
                            let _ = std::fs::remove_file(&waiting);
                        }
                        on_ui(move |studio, _, _| match result {
                            Ok(place) => say(studio, &place),
                            Err(error) => {
                                log::warn!("project file: could not publish: {error}");
                                studio.notify(&tf("project.couldNotExport", &[&error]), true);
                            }
                        });
                    });
                if !handed {
                    say(studio, &out.display().to_string());
                }
            }
        },
    );
}

/// Exports a project from the launch screen's list, without opening it.
pub fn export_recent(studio: &mut Studio, folder: String) {
    if !projects::is_project(Path::new(&folder)) {
        studio.notify(&tf("start.folderMissing", &[&folder]), true);
        return;
    }
    match projects::read_document(&folder) {
        Ok(Some(document)) => export(studio, folder, document),
        Ok(None) => studio.notify(&tf("start.folderMissing", &[&folder]), true),
        Err(error) => studio.notify(&tf("project.couldNotExport", &[&error]), true),
    }
}

/// Asks for a project file; the answer comes back as
/// [`StartMsg::ImportFrom`].
pub fn pick() {
    pick_files_then(
        t("project.openFile"),
        Some((t("project.fileFamily"), EXTENSIONS)),
        |studio, _, _, paths| {
            if let Some(path) = paths.into_iter().next() {
                studio.handle(Msg::Start(StartMsg::ImportFrom(path)));
            }
        },
    );
}

/// Unpacks `bundle` into a new folder of `location` and opens the project.
///
/// The file is checked first, on the spot, so a zip that is not a project
/// says so at once; the unpacking copies every video and takes a while, so
/// it runs on a worker. A project of the same name is never overwritten:
/// the new one takes the next free folder.
pub fn import(studio: &mut Studio, bundle: PathBuf, location: String) {
    let name = match concat_bundle::peek(&bundle) {
        Ok(name) => name,
        Err(error) => {
            studio.notify(&tf("project.couldNotImport", &[&error]), true);
            return;
        }
    };
    let location = if location.trim().is_empty() {
        PathBuf::from(home_folder("Documents")).join("Concat")
    } else {
        PathBuf::from(location.trim())
    };
    studio.notify(&tf("project.unpacking", &[&name]), false);
    spawn(
        move || {
            std::fs::create_dir_all(&location)
                .map_err(|error| format!("{}: {error}", location.display()))?;
            let folder = projects::free_folder(&location, &name);
            let target = location.join(folder);
            let unpacked = concat_bundle::unpack(&bundle, &target).map_err(|e| e.to_string())?;
            let info = projects::open(&target.to_string_lossy()).map_err(|e| {
                let _ = std::fs::remove_dir_all(&target);
                e
            })?;
            // A phone's picker copied the file into the app's own folder
            // before handing it over; that copy has done its job.
            if bundle
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|folder| folder == "Imported")
            {
                let _ = std::fs::remove_file(&bundle);
            }
            Ok::<_, String>((info, unpacked))
        },
        |studio, _, _, result| match result {
            Err(error) => {
                log::warn!("project file: could not unpack: {error}");
                studio.notify(&tf("project.couldNotImport", &[&error]), true);
            }
            Ok((info, unpacked)) => {
                let name = unpacked.name;
                match studio.open_project(info) {
                    Err(error) => studio.notify(&error, true),
                    Ok(()) if unpacked.missing.is_empty() => {
                        studio.notify(&tf("project.imported", &[&name]), false);
                    }
                    Ok(()) => studio.notify(
                        &tf("project.importedMissing", &[&name, &unpacked.missing.len()]),
                        false,
                    ),
                }
            }
        },
    );
}
