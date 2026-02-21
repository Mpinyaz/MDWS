use polars::prelude::*;
use serde_json::Value;
use std::path::PathBuf;
pub fn read_data_frame_from_csv(path_str: String) -> DataFrame {
    let file_path = PathBuf::from(path_str);

    CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(file_path))
        .expect("Failed to create CSV reader")
        .finish()
        .expect("Failed to parse CSV into DataFrame") // Handle parsing errors
}

pub fn json_to_dataframe(
    raw_json: &str,
    float_cols: &[&str],
) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let json: Value = serde_json::from_str(raw_json)?;

    // Extract the series (handling the nested InfluxQL JSON structure)
    let series_list = json["results"][0]["series"]
        .as_array()
        .ok_or("No series found. Check if the database/measurement exists.")?;
    let first_series = &series_list[0];

    let column_names: Vec<String> = first_series["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let rows = first_series["values"].as_array().ok_or("No values found")?;

    let mut polars_columns = Vec::new();

    for (col_idx, name) in column_names.iter().enumerate() {
        let col_data: Vec<Option<String>> = rows
            .iter()
            .map(|row| {
                let val = &row[col_idx];
                if val.is_null() {
                    None
                } else if let Some(s) = val.as_str() {
                    Some(s.to_string())
                } else {
                    Some(val.to_string())
                }
            })
            .collect();

        polars_columns.push(Column::from(Series::new(name.into(), col_data)));
    }

    let mut df = DataFrame::new_infer_height(polars_columns)?;

    // --- TEMPORAL PARSING ---
    // Convert the string "time" column to a proper Datetime type
    if df.column("time").is_ok() {
        df = df
            .lazy()
            .with_column(col("time").str().to_datetime(
                Some(TimeUnit::Microseconds), // InfluxDB typically uses µs/ns precision
                None,
                StrptimeOptions::default(),
                lit("raise"),
            ))
            .collect()?;
    }

    // Cast numeric fields
    for &c in float_cols {
        if df.column(c).is_ok() {
            df = df
                .lazy()
                .with_column(col(c).cast(DataType::Float64))
                .collect()?;
        }
    }

    Ok(df)
}
