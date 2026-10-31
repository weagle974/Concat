use super::*;

/// A project folder with two media files and a font, and the edit that
/// names them (one of them twice, and one that is gone).
fn project(root: &Path) -> (PathBuf, Value) {
    let folder = root.join("project");
    std::fs::create_dir_all(&folder).expect("a folder");
    let source = root.join("sources");
    std::fs::create_dir_all(&source).expect("a folder");
    std::fs::write(source.join("clip.mp4"), b"video bytes").expect("a file");
    std::fs::write(source.join("song.wav"), b"audio bytes").expect("a file");
    std::fs::write(source.join("face.ttf"), b"font bytes").expect("a file");
    std::fs::write(folder.join("preview.png"), b"png bytes").expect("a file");
    let document = json!({
        "concat": "0.2.8",
        "name": "Holiday",
        "video": { "width": 1080, "height": 1920 },
        "media": [
            { "id": "m1", "path": source.join("clip.mp4"), "name": "clip.mp4" },
            { "id": "m2", "path": source.join("song.wav"), "name": "song.wav" },
            { "id": "m3", "path": source.join("clip.mp4"), "name": "again" },
            { "id": "m4", "path": source.join("gone.mov"), "name": "gone.mov" },
        ],
        "fonts": [ { "family": "Face", "path": source.join("face.ttf") } ],
        "timelines": [ { "id": "t1" } ],
    });
    (folder, document)
}

fn paths(document: &Value, list: &str) -> Vec<String> {
    document[list]
        .as_array()
        .expect("a list")
        .iter()
        .map(|item| item["path"].as_str().expect("a path").to_owned())
        .collect()
}

#[test]
fn a_project_comes_back_with_its_files_and_its_paths_pointing_at_them() {
    let scratch = tempfile::tempdir().expect("a scratch folder");
    let (folder, document) = project(scratch.path());
    let bundle = scratch.path().join(format!("Holiday.{EXTENSION}"));

    let packed = pack(&document, &folder, &bundle).expect("it packs");
    assert_eq!(packed.files, 3, "the clip once, the song, the font");
    assert_eq!(
        packed.bytes,
        (b"video bytes".len() + b"audio bytes".len() + b"font bytes".len()) as u64
    );
    assert_eq!(packed.missing.len(), 1);
    assert!(packed.missing[0].ends_with("gone.mov"));
    assert!(
        !bundle.with_extension("part").exists(),
        "no half-written file left"
    );
    assert_eq!(peek(&bundle).expect("it is a bundle"), "Holiday");

    let target = scratch.path().join("restored");
    let unpacked = unpack(&bundle, &target).expect("it unpacks");
    assert_eq!(unpacked.name, "Holiday");
    assert_eq!(unpacked.files, 3);
    assert_eq!(unpacked.missing, packed.missing);

    let text = std::fs::read_to_string(target.join("concat.json")).expect("the edit");
    let restored: Value = serde_json::from_str(&text).expect("json");
    let media = paths(&restored, "media");
    assert_eq!(media[0], media[2], "the clip used twice is one file");
    assert_eq!(std::fs::read(&media[0]).expect("the clip"), b"video bytes");
    assert_eq!(std::fs::read(&media[1]).expect("the song"), b"audio bytes");
    assert!(media[0].starts_with(target.to_str().expect("utf8")));
    assert!(
        media[3].ends_with("gone.mov"),
        "a lost file stays lost, by its old path"
    );
    assert_eq!(
        std::fs::read(&paths(&restored, "fonts")[0]).expect("the font"),
        b"font bytes"
    );
    assert_eq!(
        restored["timelines"], document["timelines"],
        "the rest of the edit is untouched"
    );
    assert_eq!(
        std::fs::read(target.join("preview.png")).expect("the picture"),
        b"png bytes"
    );
}

#[test]
fn two_files_with_one_name_do_not_overwrite_each_other() {
    let scratch = tempfile::tempdir().expect("a scratch folder");
    let (one, two) = (scratch.path().join("a"), scratch.path().join("b"));
    std::fs::create_dir_all(&one).expect("a folder");
    std::fs::create_dir_all(&two).expect("a folder");
    std::fs::write(one.join("IMG.mp4"), b"first").expect("a file");
    std::fs::write(two.join("IMG.mp4"), b"second").expect("a file");
    let document = json!({
        "name": "Same names",
        "media": [
            { "id": "m1", "path": one.join("IMG.mp4") },
            { "id": "m2", "path": two.join("IMG.mp4") },
        ],
    });
    let bundle = scratch.path().join("same.concatproj.zip");
    pack(&document, scratch.path(), &bundle).expect("it packs");
    let target = scratch.path().join("out");
    unpack(&bundle, &target).expect("it unpacks");
    let restored: Value =
        serde_json::from_slice(&std::fs::read(target.join("concat.json")).expect("edit"))
            .expect("json");
    let media = paths(&restored, "media");
    assert_eq!(std::fs::read(&media[0]).expect("first"), b"first");
    assert_eq!(std::fs::read(&media[1]).expect("second"), b"second");
}

#[test]
fn an_edit_with_no_media_packs_and_unpacks() {
    let scratch = tempfile::tempdir().expect("a scratch folder");
    let document = json!({ "name": "Empty", "video": { "width": 1, "height": 1 } });
    let bundle = scratch.path().join("empty.zip");
    let packed = pack(&document, scratch.path(), &bundle).expect("it packs");
    assert_eq!((packed.files, packed.missing.len()), (0, 0));
    let target = scratch.path().join("out");
    let unpacked = unpack(&bundle, &target).expect("it unpacks");
    assert_eq!((unpacked.name.as_str(), unpacked.files), ("Empty", 0));
    assert!(target.join("concat.json").is_file());
}

/// A zip written by hand, with whatever entries a hostile one would have.
fn forged(path: &Path, entries: &[(&str, &[u8])]) {
    let mut zip = ZipWriter::new(File::create(path).expect("a file"));
    for (name, bytes) in entries {
        zip.start_file(*name, SimpleFileOptions::default())
            .expect("an entry");
        zip.write_all(bytes).expect("bytes");
    }
    zip.finish().expect("finished");
}

fn info(version: u64) -> Vec<u8> {
    serde_json::to_vec(&json!({ "kind": KIND, "version": version, "name": "Forged" }))
        .expect("json")
}

#[test]
fn an_entry_that_climbs_out_of_the_folder_is_never_written() {
    let scratch = tempfile::tempdir().expect("a scratch folder");
    let bundle = scratch.path().join("evil.zip");
    let document = serde_json::to_vec(&json!({
        "name": "Evil",
        "media": [ { "id": "m1", "path": "bundle:media/../../escaped.txt" } ],
    }))
    .expect("json");
    forged(
        &bundle,
        &[
            (INFO, &info(1)),
            (MANIFEST, &document),
            ("media/../../escaped.txt", b"gotcha"),
            ("../escaped2.txt", b"gotcha"),
            ("media/sub/deep.txt", b"gotcha"),
        ],
    );
    let target = scratch.path().join("inner").join("out");
    let error = unpack(&bundle, &target).expect_err("the edit names a file that was not unpacked");
    assert!(matches!(error, Error::Damaged(_)), "{error}");
    assert!(!scratch.path().join("escaped.txt").exists());
    assert!(!scratch.path().join("escaped2.txt").exists());
    assert!(!scratch.path().join("inner").join("escaped.txt").exists());
    assert!(!target.exists(), "a failed unpack leaves no folder behind");
}

#[test]
fn a_zip_that_is_not_ours_is_refused_by_name() {
    let scratch = tempfile::tempdir().expect("a scratch folder");
    let plain = scratch.path().join("plain.zip");
    forged(&plain, &[("hello.txt", b"hi")]);
    assert!(matches!(peek(&plain), Err(Error::NotABundle)));

    let text = scratch.path().join("text.zip");
    std::fs::write(&text, b"this is not a zip at all").expect("a file");
    assert!(matches!(peek(&text), Err(Error::Zip(_))));
    assert!(matches!(
        unpack(&text, &scratch.path().join("out")),
        Err(Error::Zip(_))
    ));
    assert!(!scratch.path().join("out").exists());
}

#[test]
fn a_bundle_from_a_newer_app_says_so() {
    let scratch = tempfile::tempdir().expect("a scratch folder");
    let bundle = scratch.path().join("future.zip");
    forged(&bundle, &[(INFO, &info(VERSION + 1)), (MANIFEST, b"{}")]);
    assert!(matches!(peek(&bundle), Err(Error::TooNew(2))));
}

#[test]
fn a_bundle_without_its_edit_is_damaged_and_leaves_nothing() {
    let scratch = tempfile::tempdir().expect("a scratch folder");
    let bundle = scratch.path().join("headless.zip");
    forged(&bundle, &[(INFO, &info(1))]);
    let target = scratch.path().join("out");
    assert!(matches!(unpack(&bundle, &target), Err(Error::Damaged(_))));
    assert!(!target.exists());
}

#[test]
fn a_folder_that_holds_something_is_not_unpacked_into() {
    let scratch = tempfile::tempdir().expect("a scratch folder");
    let document = json!({ "name": "X" });
    let bundle = scratch.path().join("x.zip");
    pack(&document, scratch.path(), &bundle).expect("it packs");
    let target = scratch.path().join("busy");
    std::fs::create_dir_all(&target).expect("a folder");
    std::fs::write(target.join("mine.txt"), b"keep").expect("a file");
    assert!(unpack(&bundle, &target).is_err());
    assert_eq!(
        std::fs::read(target.join("mine.txt")).expect("kept"),
        b"keep"
    );
}

#[test]
fn file_names_are_made_safe_without_losing_what_they_are() {
    assert_eq!(safe_name("a/b\\c:d.mp4"), "a_b_c_d.mp4");
    assert_eq!(safe_name("..hidden"), "hidden");
    assert_eq!(safe_name(""), "file");
    let long = format!("{}.mp4", "x".repeat(300));
    let cut = safe_name(&long);
    assert!(cut.len() <= 90 && cut.ends_with(".mp4"), "{cut}");
    assert!(is_media_entry("media/1-a.mp4"));
    for bad in [
        "media/",
        "media/..",
        "media/a/b",
        "media/a\\b",
        "../media/a",
        "other/a",
        "media/.x",
        "media/c:x",
    ] {
        assert!(!is_media_entry(bad), "{bad}");
    }
}
