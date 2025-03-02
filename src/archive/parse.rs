use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use crate::{
    error::{GeneralError, InvalidArgumentError},
    plugin::has_ascii_extension,
};

use super::{ba2, bsa};

pub fn assets_in_archives(archive_paths: &[PathBuf]) -> BTreeMap<u64, BTreeSet<u64>> {
    let mut archive_assets: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();

    for archive_path in archive_paths {
        log::trace!(
            "Getting assets loaded from the Bethesda archive at \"{}\"",
            archive_path.display()
        );

        let assets = match get_assets_in_archive(archive_path) {
            Ok(a) => a,
            Err(e) => {
                log::error!(
                    "Encountered an error while trying to read the Bethesda archive at \"{}\": {}",
                    archive_path.display(),
                    e
                );
                continue;
            }
        };

        let warn_on_hash_collisions = should_warn_on_hash_collisions(archive_path);

        for (folder_hash, file_hashes) in assets {
            let entry_file_hashes = archive_assets.entry(folder_hash).or_default();

            for file_hash in file_hashes {
                if !entry_file_hashes.insert(file_hash) && warn_on_hash_collisions {
                    log::warn!(
                        "The folder and file with hashes {:x} and {:x} in \"{}\" are present in another Bethesda archive.",
                        folder_hash,
                        file_hash,
                        archive_path.display()
                    );
                }
            }
        }
    }

    archive_assets
}

fn should_warn_on_hash_collisions(archive_path: &Path) -> bool {
    if !has_ascii_extension(archive_path, "ba2") {
        return true;
    }

    let filename = archive_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();

    filename.starts_with("fallout4 - ") || filename.starts_with("dlcultrahighresolution - ")
}

fn get_assets_in_archive(
    archive_path: &Path,
) -> Result<BTreeMap<u64, BTreeSet<u64>>, GeneralError> {
    let mut reader = BufReader::new(File::open(archive_path)?);

    let mut type_id: [u8; 4] = [0; 4];
    reader.read_exact(&mut type_id)?;

    match type_id {
        bsa::TYPE_ID => bsa::read_assets(reader),
        ba2::TYPE_ID => ba2::read_assets(reader),
        _ => Err(InvalidArgumentError {
            message: format!(
                "Bethesda archive at \"{}\" has an unrecognised type ID",
                archive_path.display()
            ),
        }
        .into()),
    }
}

pub(super) fn to_u32(bytes: &[u8]) -> u32 {
    let array =
        <[u8; 4]>::try_from(&bytes[..4]).expect("Bytes slice is large enough to hold a u32");
    u32::from_le_bytes(array)
}

pub(super) fn to_u64(bytes: &[u8]) -> u64 {
    let array =
        <[u8; 8]>::try_from(&bytes[..4]).expect("Bytes slice is large enough to hold a u64");
    u64::from_le_bytes(array)
}

pub(super) fn to_usize(size: u32) -> usize {
    usize::try_from(size).expect("usize can hold a u32")
}
