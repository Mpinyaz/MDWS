use polars::prelude::*;
use std::fs::File;
fn main() {
    let lf = LazyCsvReader::new("large_file.csv");
}
