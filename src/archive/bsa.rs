use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    io::BufRead,
};

use crate::error::{GeneralError, InvalidArgumentError};

use super::parse::{to_u32, to_u64, to_usize};

pub(super) const TYPE_ID: [u8; 4] = *b"BSA\0";
const HEADER_SIZE: usize = 36;
const FILE_RECORD_SIZE: usize = 16;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Header {
    type_id: [u8; 4],
    version: u32,
    records_offset: u32,
    archive_flags: u32,
    folder_count: u32,
    total_file_count: u32,
    total_folder_names_length: u32,
    total_file_names_length: u32,
    content_type_flags: u32,
}

impl TryFrom<[u8; HEADER_SIZE - TYPE_ID.len()]> for Header {
    type Error = InvalidArgumentError;

    fn try_from(value: [u8; HEADER_SIZE - TYPE_ID.len()]) -> Result<Self, Self::Error> {
        let header = Self {
            type_id: TYPE_ID,
            version: to_u32(&value),
            records_offset: to_u32(&value[4..]),
            archive_flags: to_u32(&value[8..]),
            folder_count: to_u32(&value[12..]),
            total_file_count: to_u32(&value[16..]),
            total_folder_names_length: to_u32(&value[20..]),
            total_file_names_length: to_u32(&value[24..]),
            content_type_flags: to_u32(&value[28..]),
        };

        if header.records_offset != 36 {
            return Err(InvalidArgumentError {
                message: format!(
                    "BSA file has an invalid records offset value: {}",
                    header.records_offset
                ),
            });
        }

        if (header.archive_flags & 0x40) != 0 {
            return Err(InvalidArgumentError {
                message: "BSA file uses big-endian numbers".into(),
            });
        }

        Ok(header)
    }
}

struct FolderRecord {
    name_hash: u64,
    file_count: u32,
    file_records_offset: u32,
}

// Also used for v104 BSAs.
mod v103 {
    use crate::archive::parse::{to_u32, to_u64};

    use super::FolderRecord;

    pub(super) const FOLDER_RECORD_SIZE: usize = 16;

    pub(super) fn read_folder_record(value: &[u8]) -> FolderRecord {
        assert!(value.len() >= FOLDER_RECORD_SIZE);

        FolderRecord {
            name_hash: to_u64(value),
            file_count: to_u32(&value[8..]),
            file_records_offset: to_u32(&value[12..]),
        }
    }
}

mod v105 {
    use crate::archive::parse::{to_u32, to_u64};

    use super::FolderRecord;

    pub(super) const FOLDER_RECORD_SIZE: usize = 24;

    pub(super) fn read_folder_record(value: &[u8]) -> FolderRecord {
        assert!(value.len() >= FOLDER_RECORD_SIZE);

        FolderRecord {
            name_hash: to_u64(value),
            file_count: to_u32(&value[8..]),
            file_records_offset: to_u32(&value[16..]),
        }
    }
}

pub(super) fn read_assets<T: BufRead>(
    mut reader: T,
) -> Result<BTreeMap<u64, BTreeSet<u64>>, GeneralError> {
    let mut header_buffer = [0; HEADER_SIZE - TYPE_ID.len()];

    reader.read_exact(&mut header_buffer)?;

    let header = Header::try_from(header_buffer)?;

    match header.version {
        103 | 104 => read_assets_with_header::<T, { v103::FOLDER_RECORD_SIZE }>(
            reader,
            &header,
            v103::read_folder_record,
        ),
        105 => read_assets_with_header::<T, { v105::FOLDER_RECORD_SIZE }>(
            reader,
            &header,
            v105::read_folder_record,
        ),
        _ => Err(InvalidArgumentError {
            message: format!("BSA file has an unrecognised version: {}", header.version),
        }
        .into()),
    }
}

fn read_assets_with_header<T: BufRead, const U: usize>(
    mut reader: T,
    header: &Header,
    read_folder_record: impl Fn(&[u8]) -> FolderRecord,
) -> Result<BTreeMap<u64, BTreeSet<u64>>, GeneralError> {
    let mut folders_buffer: Vec<u8> = vec![0; U * to_usize(header.folder_count)];

    reader.read_exact(folders_buffer.as_mut_slice())?;

    let file_records_size = to_usize(header.folder_count)
        + to_usize(header.total_folder_names_length)
        + to_usize(header.total_file_count) * FILE_RECORD_SIZE;

    let mut file_records_buffer: Vec<u8> = vec![0; file_records_size];

    reader.read_exact(file_records_buffer.as_mut_slice())?;

    let folder_record_offset_baseline =
        HEADER_SIZE + folders_buffer.len() + to_usize(header.total_file_names_length);

    let mut assets = BTreeMap::new();
    for chunk in folders_buffer.chunks_exact(U) {
        let folder_record = read_folder_record(chunk);

        let entry = assets.entry(folder_record.name_hash);
        if let Entry::Occupied(_) = entry {
            return Err(InvalidArgumentError {
                message: format!(
                    "Unexpected collision for folder name hash {:x}",
                    folder_record.name_hash
                ),
            }
            .into());
        }

        let file_records_offset = if (header.archive_flags & 0x1) == 0 {
            to_usize(folder_record.file_records_offset) - folder_record_offset_baseline
        } else {
            let folder_name_length_offset =
                to_usize(folder_record.file_records_offset) - folder_record_offset_baseline;

            if let Some(folder_name_length) = file_records_buffer.get(folder_name_length_offset) {
                folder_name_length_offset + 1 + to_usize(u32::from(*folder_name_length))
            } else {
                return Err(InvalidArgumentError {
                    message: "BSA file contains an invalid folder name length offset".into(),
                }
                .into());
            }
        };

        let file_records_buffer = match file_records_buffer.get(file_records_offset..) {
            Some(s) => s,
            None => {
                return Err(InvalidArgumentError {
                    message: "BSA file contains an invalid file records offset".into(),
                }
                .into());
            }
        };

        let file_hashes: &mut BTreeSet<u64> = entry.or_default();

        for file_chunk in file_records_buffer
            .chunks_exact(FILE_RECORD_SIZE)
            .take(to_usize(folder_record.file_count))
        {
            let file_hash = to_u64(file_chunk);

            if !file_hashes.insert(file_hash) {
                return Err(InvalidArgumentError { message: format!("Unexpected collision for file name hash {:x} in set for folder name hash {:x}", file_hash, folder_record.name_hash)}.into());
            }
        }
    }

    Ok(assets)
}
