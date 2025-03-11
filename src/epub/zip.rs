use std::fs::File;
use std::io::{Read, Seek};
use std::mem::{size_of, transmute};
use std::ops::Range;
use std::os::unix::fs::FileExt;

use miniz_oxide::inflate::decompress_to_vec_with_limit;

use crate::error::{OrDie, 即死, 死};

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
struct LocalFileHeader {
    signature: [u8; 4],
    version_needed: [u8; 2],
    general_purpose: [u8; 2],
    compression_method: [u8; 2],
    last_mod_time: [u8; 2],
    last_mod_date: [u8; 2],
    crc32: [u8; 4],
    compressed_size: [u8; 4],
    uncompressed_size: [u8; 4],
    filename_len: [u8; 2],
    extra_field_len: [u8; 2],
}

const LFH_SIGNATURE: u32 = 0x04034b50;
const CDFH_SIGNATURE: u32 = 0x02014b50;
const LFH_SIZE: usize = size_of::<LocalFileHeader>();

pub struct FileIter<'a> {
    file: &'a mut File,
}

impl<'a> FileIter<'a> {
    pub fn new(file: &'a mut File) -> Self {
        Self { file }
    }

    fn read_zip_header(&mut self) -> Option<DeflatedFile> {
        let mut buf = [0; LFH_SIZE];
        self.file.read_exact(&mut buf).or_(死!());

        let header: &LocalFileHeader = unsafe { transmute(&buf) };

        let signature = u32::from_le_bytes(header.signature);
        if signature != LFH_SIGNATURE {
            if signature == CDFH_SIGNATURE {
                return None;
            }
            即死!("invalid zip file");
        }

        let filename_len = u16::from_le_bytes(header.filename_len) as usize;
        let mut filename = vec![0; filename_len];
        self.file.read_exact(&mut filename).or_(死!());
        let filename = String::from_utf8(filename).or_(死!());

        let extra_fields_len = u16::from_le_bytes(header.extra_field_len).to_le() as i64;
        self.file.seek_relative(extra_fields_len).or_(死!());

        let compressed_size = u32::from_le_bytes(header.compressed_size) as u64;
        let current_pos = self.file.stream_position().or_(死!());
        let range = current_pos..(current_pos + compressed_size);

        let uncompressed_size = u32::from_le_bytes(header.uncompressed_size) as usize;
        self.file.seek_relative(compressed_size as i64).or_(死!());

        Some(DeflatedFile {
            name: filename,
            range,
            size: uncompressed_size,
        })
    }
}

impl Iterator for FileIter<'_> {
    type Item = DeflatedFile;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_zip_header()
    }
}

#[derive(Debug, Clone)]
pub struct DeflatedFile {
    pub name: String,
    range: Range<u64>,
    size: usize,
}

impl DeflatedFile {
    pub fn extract_string(&self, file: &mut File) -> String {
        let len = (self.range.end - self.range.start) as usize;
        let mut deflate_bytes = vec![0; len];
        file.read_exact_at(&mut deflate_bytes, self.range.start)
            .or_(死!());
        let contents = decompress_to_vec_with_limit(&deflate_bytes, self.size).or_(死!());
        String::from_utf8(contents).or_(死!())
    }
}
