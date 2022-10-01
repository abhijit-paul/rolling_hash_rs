use std::fs::File;
use std::io::{BufReader, Read, Result};
use std::path::Path;

pub fn read_file_to_buffer(reader: &mut BufReader<&File>) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn read_handler(input_path: &Path) -> Result<File> {
    match File::open(input_path) {
        Ok(file) => Ok(file),
        Err(err) => {
            eprintln!(
                "cannot open file for reading: {:?}, error: {}",
                input_path, err
            );
            Err(err)
        }
    }
}

pub fn write_handler(output_path: &Path) -> Result<File> {
    match File::create(output_path) {
        Ok(file) => Ok(file),
        Err(err) => {
            eprintln!(
                "cannot open file for writing: {:?}, error: {}",
                output_path, err
            );
            Err(err)
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_read_handler() {
        let new_file: File = read_handler(Path::new("data/new.txt")).unwrap();
        assert_eq!(3096, new_file.metadata().unwrap().len());
    }
}