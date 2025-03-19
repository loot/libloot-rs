use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use super::error::{ArchiveParsingError, ArchivePathParsingError};
use crate::{
    logging::{self, format_details},
    plugin::has_ascii_extension,
};

use super::{ba2, bsa};

pub fn assets_in_archives(archive_paths: &[PathBuf]) -> BTreeMap<u64, BTreeSet<u64>> {
    let mut archive_assets: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();

    for archive_path in archive_paths {
        logging::trace!(
            "Getting assets loaded from the Bethesda archive at \"{}\"",
            archive_path.display()
        );

        let assets = match get_assets_in_archive(archive_path) {
            Ok(a) => a,
            Err(e) => {
                logging::error!(
                    "Encountered an error while trying to read the Bethesda archive at \"{}\": {}",
                    archive_path.display(),
                    format_details(&e)
                );
                continue;
            }
        };

        let warn_on_hash_collisions = should_warn_on_hash_collisions(archive_path);

        for (folder_hash, file_hashes) in assets {
            let entry_file_hashes = archive_assets.entry(folder_hash).or_default();

            for file_hash in file_hashes {
                if !entry_file_hashes.insert(file_hash) && warn_on_hash_collisions {
                    logging::warn!(
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
) -> Result<BTreeMap<u64, BTreeSet<u64>>, ArchivePathParsingError> {
    let file = File::open(archive_path)
        .map_err(|e| ArchivePathParsingError::from_io_error(archive_path.into(), e))?;
    let mut reader = BufReader::new(file);

    let mut type_id: [u8; 4] = [0; 4];
    reader
        .read_exact(&mut type_id)
        .map_err(|e| ArchivePathParsingError::from_io_error(archive_path.into(), e))?;

    match type_id {
        bsa::TYPE_ID => bsa::read_assets(reader)
            .map_err(|e| ArchivePathParsingError::new(archive_path.into(), e)),
        ba2::TYPE_ID => ba2::read_assets(reader)
            .map_err(|e| ArchivePathParsingError::new(archive_path.into(), e)),
        _ => Err(ArchivePathParsingError::new(
            archive_path.into(),
            ArchiveParsingError::UnsupportedArchiveTypeId(type_id),
        )),
    }
}

pub(super) fn to_u32(bytes: &[u8]) -> u32 {
    let array =
        <[u8; 4]>::try_from(&bytes[..4]).expect("Bytes slice is large enough to hold a u32");
    u32::from_le_bytes(array)
}

pub(super) fn to_u64(bytes: &[u8]) -> u64 {
    let array =
        <[u8; 8]>::try_from(&bytes[..8]).expect("Bytes slice is large enough to hold a u64");
    u64::from_le_bytes(array)
}

pub(super) fn to_usize(size: u32) -> usize {
    usize::try_from(size).expect("usize can hold a u32")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_assets_in_archive {
        use std::{
            hash::{DefaultHasher, Hash, Hasher},
            io::SeekFrom,
        };

        use rstest::rstest;
        use tempfile::tempdir;

        use super::*;

        fn hash<T: Hash>(value: T) -> u64 {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        }

        #[test]
        fn should_error_if_file_cannot_be_opened() {
            let path = Path::new("./invalid.bsa");
            assert!(get_assets_in_archive(path).is_err());
        }

        #[test]
        fn should_support_v103_bsas() {
            let path = Path::new("./testing-plugins/Oblivion/Data/Blank.bsa");
            let assets = get_assets_in_archive(path).unwrap();

            let files_count: usize = assets.values().map(|v| v.len()).sum();

            let expected_key = 0;
            assert_eq!(1, assets.len());
            assert_eq!(1, files_count);
            assert_eq!(expected_key, *assets.first_key_value().unwrap().0);
            assert_eq!(1, assets.get(&expected_key).unwrap().len());
            assert_eq!(
                0x4670B6836C077365,
                *assets.get(&expected_key).unwrap().first().unwrap()
            );
        }

        #[test]
        fn should_support_v104_bsas() {
            let path = Path::new("./testing-plugins/Skyrim/Data/Blank.bsa");
            let assets = get_assets_in_archive(path).unwrap();

            let files_count: usize = assets.values().map(|v| v.len()).sum();

            let expected_key = 0x2E01002E;
            assert_eq!(1, assets.len());
            assert_eq!(1, files_count);
            assert_eq!(expected_key, *assets.first_key_value().unwrap().0);
            assert_eq!(1, assets.get(&expected_key).unwrap().len());
            assert_eq!(
                0x4670B6836C077365,
                *assets.get(&expected_key).unwrap().first().unwrap()
            );
        }

        #[test]
        fn should_support_v105_bsas() {
            let path = Path::new("./testing-plugins/SkyrimSE/Data/Blank.bsa");
            let assets = get_assets_in_archive(path).unwrap();

            let files_count: usize = assets.values().map(|v| v.len()).sum();

            let expected_key = 0xB68102C964176E73;
            assert_eq!(1, assets.len());
            assert_eq!(1, files_count);
            assert_eq!(expected_key, *assets.first_key_value().unwrap().0);
            assert_eq!(1, assets.get(&expected_key).unwrap().len());
            assert_eq!(
                0x4670B6836C077365,
                *assets.get(&expected_key).unwrap().first().unwrap()
            );
        }

        #[test]
        fn should_support_general_ba2s() {
            let path = Path::new("./testing-plugins/Fallout 4/Data/Blank - Main.ba2");
            let assets = get_assets_in_archive(path).unwrap();

            let files_count: usize = assets.values().map(|v| v.len()).sum();

            let expected_key = hash("dev\\git\\testing-plugins".as_bytes());
            let expected_file_hash = hash("license.txt".as_bytes());

            assert_eq!(1, assets.len());
            assert_eq!(1, files_count);

            let (key, value) = assets.first_key_value().unwrap();
            assert_eq!(expected_key, *key);
            assert_eq!(1, value.len());
            assert_eq!(expected_file_hash, *value.first().unwrap());
        }

        #[test]
        fn should_support_texture_ba2s() {
            let path = Path::new("./testing-plugins/Fallout 4/Data/Blank - Textures.ba2");
            let assets = get_assets_in_archive(path).unwrap();

            let files_count: usize = assets.values().map(|v| v.len()).sum();

            let expected_key = hash("dev\\git\\testing-plugins".as_bytes());
            let expected_file_hash = hash("blank.dds".as_bytes());

            assert_eq!(1, assets.len());
            assert_eq!(1, files_count);

            let (key, value) = assets.first_key_value().unwrap();
            assert_eq!(expected_key, *key);
            assert_eq!(1, value.len());
            assert_eq!(expected_file_hash, *value.first().unwrap());
        }

        #[rstest]
        fn should_support_ba2_versions(#[values(1, 2, 3, 7, 8)] version: u32) {
            use std::io::{Seek, Write};

            let tmp_dir = tempdir().unwrap();
            let path = tmp_dir.path().join("test.ba2");

            std::fs::copy("./testing-plugins/Fallout 4/Data/Blank - Main.ba2", &path).unwrap();

            {
                let mut file = File::options().write(true).open(&path).unwrap();
                file.seek(SeekFrom::Start(4)).unwrap();
                file.write_all(&version.to_le_bytes()).unwrap();
            }

            let assets = get_assets_in_archive(&path).unwrap();
            assert!(!assets.is_empty());
        }
    }

    mod assets_in_archives {
        use super::*;

        #[test]
        fn should_skip_files_that_cannot_be_read() {
            let paths = [
                PathBuf::from("invalid.bsa"),
                PathBuf::from("./testing-plugins/Skyrim/Data/Blank.bsa"),
            ];

            let assets = assets_in_archives(&paths);

            let files_count: usize = assets.values().map(|v| v.len()).sum();

            assert_eq!(1, assets.len());
            assert_eq!(1, files_count);

            let (key, value) = assets.first_key_value().unwrap();
            assert_eq!(0x2E01002E, *key);
            assert_eq!(1, value.len());
            assert_eq!(0x4670B6836C077365, *value.first().unwrap());
        }

        #[test]
        fn should_combine_assets_from_each_loaded_archive() {
            let paths = [
                PathBuf::from("./testing-plugins/Oblivion/Data/Blank.bsa"),
                PathBuf::from("./testing-plugins/Skyrim/Data/Blank.bsa"),
                PathBuf::from("./testing-plugins/SkyrimSE/Data/Blank.bsa"),
            ];

            let assets = assets_in_archives(&paths);

            let files_count: usize = assets.values().map(|v| v.len()).sum();

            assert_eq!(3, assets.len());
            assert_eq!(3, files_count);

            let value = assets.get(&0).unwrap();
            assert_eq!(1, value.len());
            assert_eq!(0x4670B6836C077365, *value.first().unwrap());

            let value = assets.get(&0x2E01002E).unwrap();
            assert_eq!(1, value.len());
            assert_eq!(0x4670B6836C077365, *value.first().unwrap());

            let value = assets.get(&0xB68102C964176E73).unwrap();
            assert_eq!(1, value.len());
            assert_eq!(0x4670B6836C077365, *value.first().unwrap());
        }
    }
}
