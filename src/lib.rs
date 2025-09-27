use parquet::file::reader::{FileReader, SerializedFileReader};
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

fn parquet_file_to_row_count<P: AsRef<Path>>(
    file_path: P,
) -> Result<i64, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = SerializedFileReader::new(file)?;
    let metadata = reader.metadata();
    let row_count = metadata.file_metadata().num_rows();
    Ok(row_count)
}

pub fn count_rows_from_lines<I>(mut lines: I) -> Result<i64, Box<dyn std::error::Error>>
where
    I: Iterator<Item = io::Result<String>>,
{
    lines.try_fold(0, |acc, line_result| {
        let line = line_result?;
        let path = Path::new(&line);
        if path.is_file() && path.extension().is_some_and(|ext| ext == "parquet") {
            let count = parquet_file_to_row_count(path)?;
            Ok(acc + count)
        } else {
            Ok(acc)
        }
    })
}

pub fn count_rows<P: AsRef<Path>>(path: P) -> Result<i64, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    if path.is_dir() {
        let dir_iter = fs::read_dir(path)?.map(|entry_result| {
            entry_result.map(|entry| entry.path().to_string_lossy().into_owned())
        });
        count_rows_from_lines(dir_iter)
    } else if path.is_file() {
        parquet_file_to_row_count(path)
    } else {
        Err("Path is not a file or a directory".into())
    }
}

pub fn count_rows_from_stdin() -> Result<i64, Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let lines = stdin.lock().lines();
    count_rows_from_lines(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Arc;

    use arrow::array::Int32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::arrow_writer::ArrowWriter;
    use parquet::file::properties::WriterProperties;

    fn create_test_parquet_file(path: &str) {
        let schema = Schema::new(vec![Field::new("id", DataType::Int32, false)]);
        let batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![Arc::new(Int32Array::from(vec![1, 2, 3]))],
        )
        .unwrap();

        let file = fs::File::create(path).unwrap();
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props)).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
    }

    #[test]
    fn it_works() {
        let test_file = "test.parquet";
        create_test_parquet_file(test_file);
        let row_count = count_rows(test_file).unwrap();
        assert_eq!(row_count, 3);
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_count_rows_in_directory() {
        let test_dir = "test_dir";
        fs::create_dir(test_dir).unwrap();
        let test_file1 = "test_dir/test1.parquet";
        let test_file2 = "test_dir/test2.parquet";
        create_test_parquet_file(test_file1);
        create_test_parquet_file(test_file2);

        let row_count = count_rows(test_dir).unwrap();
        assert_eq!(row_count, 6);

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_count_rows_from_lines() {
        let test_dir = "test_lines_dir";
        fs::create_dir(test_dir).unwrap();
        let test_file1 = "test_lines_dir/test1.parquet";
        let test_file2 = "test_lines_dir/test2.parquet";
        let non_parquet_file = "test_lines_dir/test3.txt";
        create_test_parquet_file(test_file1);
        create_test_parquet_file(test_file2);
        fs::write(non_parquet_file, "hello").unwrap();

        let lines = vec![
            Ok(test_file1.to_string()),
            Ok(test_file2.to_string()),
            Ok(non_parquet_file.to_string()),
            Ok("non_existent_file.parquet".to_string()),
        ];

        let row_count = count_rows_from_lines(lines.into_iter()).unwrap();
        assert_eq!(row_count, 6);

        fs::remove_dir_all(test_dir).unwrap();
    }
}
