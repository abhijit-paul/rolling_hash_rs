use std::cmp::PartialEq;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result};

use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};

use super::file_io::read_file_to_buffer;
use super::signature::{
    chunk_sha256_hash, pointer_at_last_chunk, BlockChunkHashes, FileChunkSignature,
};
use super::window_checksum::RollingWindow;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerifyMatch {
    Match(u32),
    NoMatch(Vec<u8>),
}

// Generate diff file based on signature file and contents of modified text file
pub fn write_diff_file(signature_file: &File, new_file: &File, diff_file: &mut File) -> Result<()> {
    let signature_buf = BufReader::new(signature_file);
    let signature: FileChunkSignature = deserialize_from(signature_buf).unwrap();
    let chunk_size = signature.block_chunk_size as usize;
    let mut new_file_reader = BufReader::new(new_file);
    let mut file_buf = read_file_to_buffer(&mut new_file_reader)?;

    let diff = generate_diff(&mut file_buf, &signature, chunk_size);

    let mut diff_writer = BufWriter::new(diff_file);
    serialize_into(&mut diff_writer, &diff).unwrap();

    Ok(())
}

fn match_index_and_checksum<'a>(
    signature: &'a FileChunkSignature,
    index_hash: u32,
    chunk: &[u8],
) -> Option<&'a BlockChunkHashes> {
    if let Some(hashes) = signature.block_chunk_hashes(&index_hash) {
        let sha256_checksum_hash = chunk_sha256_hash(chunk);
        hashes.iter().find(|h| h.hash == sha256_checksum_hash)
    } else {
        None
    }
}

// Generates diff based on for file buffer, signature file and file chunk size
pub fn generate_diff(
    new_file_buffer: &mut Vec<u8>,
    signature: &FileChunkSignature,
    chunk_size: usize,
) -> Vec<VerifyMatch> {
    let mut match_verifier: Vec<VerifyMatch> = Vec::new();
    loop {
        // De-structure vector buffer to array chunk
        let chunk = if chunk_size <= new_file_buffer.len() {
            &new_file_buffer[..chunk_size]
        } else {
            &new_file_buffer[..new_file_buffer.len()]
        };

        let mut actual_chunk_size = chunk.len();
        if actual_chunk_size == 0 {
            break;
        }

        // Calculate rolling window check-sum hash
        let mut rolling_sum = RollingWindow::generate();
        rolling_sum.add_bytes_at_end(chunk);
        let index_hash = rolling_sum.sha256_digest();

        // Verify if checksum of pattern and current window matches.
        // If these two checksums don't match, move the window
        if let Some(hash) = match_index_and_checksum(signature, index_hash, chunk) {
            match_verifier.push(VerifyMatch::Match(hash.index));

            if pointer_at_last_chunk(actual_chunk_size, new_file_buffer.len()) {
                break;
            }
            // Prepare buffer for next iteration
            new_file_buffer.drain(..actual_chunk_size);
            continue;
        }

        // In case the checksum of pattern and current window doesn't match,
        // run rolling window
        let mut diff_bytes: Vec<u8> = Vec::new();
        loop {
            let mut buf_len = new_file_buffer.len();
            let mut next: Option<u8> = None;
            if !pointer_at_last_chunk(actual_chunk_size, buf_len) {
                next = Some(new_file_buffer[chunk_size]);
            }
            if buf_len > 0 {
                let prev = new_file_buffer.remove(0);
                buf_len = new_file_buffer.len();
                diff_bytes.push(prev);
                rolling_sum.roll_window(prev, next);
                let index_hash = rolling_sum.sha256_digest();
                let chunk = if chunk_size < buf_len {
                    &new_file_buffer[..chunk_size]
                } else {
                    &new_file_buffer[..buf_len]
                };
                actual_chunk_size = chunk.len();

                if let Some(hash) = match_index_and_checksum(signature, index_hash, chunk) {
                    match_verifier.push(VerifyMatch::NoMatch(diff_bytes));
                    match_verifier.push(VerifyMatch::Match(hash.index));

                    new_file_buffer.drain(..actual_chunk_size);
                    break;
                }
            } else {
                if !diff_bytes.is_empty() {
                    match_verifier.push(VerifyMatch::NoMatch(diff_bytes));
                }
                break;
            }
        }
    }
    match_verifier
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::handlers::file_io::read_handler;
    use std::path::Path;

    #[test]
    pub fn test_generate_diff() {
        let signature_file = read_handler(Path::new("data/signature")).unwrap();
        let signature_buf = BufReader::new(signature_file);
        let signature: FileChunkSignature = deserialize_from(signature_buf).unwrap();
        let chunk_size = signature.block_chunk_size;

        let new_file = read_handler(Path::new("data/new.txt")).unwrap();
        let mut new_file_reader = BufReader::new(&new_file);
        let mut buffer = read_file_to_buffer(&mut new_file_reader).unwrap();

        let diff = generate_diff(&mut buffer, &signature, chunk_size as usize);

        let expected_diff_file = read_handler(Path::new("data/diff")).unwrap();
        let expected_diff_reader = BufReader::new(expected_diff_file);

        let expected_diff: Vec<VerifyMatch> = deserialize_from(expected_diff_reader).unwrap();

        assert_eq!(expected_diff, diff);
    }
}
