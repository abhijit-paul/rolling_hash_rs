use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result};

use bincode::serialize_into;
use hmac_sha256::Hash as Sha256Hash;
use serde::{Deserialize, Serialize};

use crate::handlers::{file_io, window_checksum};

// Signature of input file
#[derive(Debug, Serialize, Deserialize)]
pub struct FileChunkSignature {
    pub block_chunk_size: u32,

    // Rolling checksum requires a store checksum based hash to avoid collision
    // But it is easy to calculate hash based on index.
    // This weaker hash is used while shifting the rolling window
    // Hence both hashes are required.
    // This stores a mapping of index based hash to the sha256 based hash
    pub checksum_map: HashMap<u32, Vec<BlockChunkHashes>>,
}

impl FileChunkSignature {
    // Evaluate hash checksum for index based checksum

    pub fn block_chunk_hashes(&self, key: &u32) -> Option<&Vec<BlockChunkHashes>> {
        self.checksum_map.get(key)
    }
}

// File block chunk has two hash as discussed above.
// This structure stores both index based hash and SHA 256 checksum based hash
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockChunkHashes {
    pub index: u32,
    pub hash: [u8; 32],
}

pub fn pointer_at_last_chunk(chunk_len: usize, buf_len: usize) -> bool {
    chunk_len == buf_len
}

// Get signature for given buffer and chunk size
pub fn get_signature(buffer: &mut Vec<u8>, block_size: u32) -> FileChunkSignature {
    let mut signature = FileChunkSignature {
        block_chunk_size: block_size,
        checksum_map: HashMap::new(),
    };

    let chunk_size = block_size as usize;
    let mut chunk_index = 0u32;

    loop {
        // Extract block of chunk from buffer
        let buffer_length = buffer.len();
        let block_chunk: &[u8] = if chunk_size < buffer_length {
            &buffer[..buffer_length]
        } else {
            &buffer[..chunk_size]
        };

        let chunk_len = block_chunk.len();
        if chunk_len == 0 {
            break;
        }

        let index_hash = window_checksum::rolling_window_checksum(block_chunk);

        let sha256_hash = chunk_sha256_hash(block_chunk);

        // Add entry to signature table
        let chunk_hashes = signature
            .checksum_map
            .entry(index_hash)
            .or_insert_with(Vec::new);

        chunk_hashes.push(BlockChunkHashes {
            index: chunk_index,
            hash: sha256_hash,
        });

        if pointer_at_last_chunk(chunk_len, buffer.len()) {
            break;
        }
        // Prepare buffer for next iteration
        buffer.drain(..chunk_len);
        chunk_index += 1;
    }
    signature
}

// Algorithm derived from https://fossies.org/linux/rdiff-backup/src/rdiff_backup/Rdiff.py
fn find_blocksize(file_length: u64) -> u32 {
    if file_length <= 4096 {
        64
    } else {
        ((file_length as f64).sqrt() / 16.0).round() as u32 * 16
    }
}

// Get signature for given input file and write the binary in a file
pub fn write_signature_file(input_file: &File, signature_file: &mut File) -> Result<()> {
    let file_len_res = input_file.metadata().map(|m| m.len());
    let chunk_size = match file_len_res {
        Ok(file_len) => find_blocksize(file_len),
        Err(_) => 500, // Use default block chunk size of 500 if file metadata doesn't have length info
    };

    let mut input_file_buf = file_io::read_file_to_buffer(&mut BufReader::new(input_file))?;
    let signature = get_signature(&mut input_file_buf, chunk_size);
    let mut signature_writer = BufWriter::new(signature_file);

    serialize_into(&mut signature_writer, &signature).unwrap();
    Ok(())
}

// Calculates SHA 256 Hash
pub fn chunk_sha256_hash(chunk: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256Hash::new();
    hasher.update(&chunk);
    let sha256_hash: [u8; 32] = hasher.finalize();
    sha256_hash
}
