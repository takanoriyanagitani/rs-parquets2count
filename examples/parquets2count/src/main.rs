use std::error::Error;
use std::process::ExitCode;

fn stdin2filenames2parquet2counts2total() -> Result<i64, Box<dyn Error>> {
    rs_parquets2count::count_rows_from_stdin()
}

fn sub() -> Result<(), Box<dyn Error>> {
    let tot: i64 = stdin2filenames2parquet2counts2total()?;
    println!("{tot}");
    Ok(())
}

fn main() -> ExitCode {
    match sub() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
