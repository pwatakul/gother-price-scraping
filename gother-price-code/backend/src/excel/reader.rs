//! Excel Reader
//!
//! Reads hotel data from Excel files.

use calamine::{open_workbook_auto_from_rs, Data, Reader};
use std::io::Cursor;

use crate::error::AppError;
use crate::models::HotelImportData;

/// Excel file reader
pub struct ExcelReader;

impl ExcelReader {
    /// Read hotels from Excel file bytes
    pub fn read_hotels(data: &[u8]) -> Result<Vec<HotelImportData>, AppError> {
        let cursor = Cursor::new(data);
        let mut workbook = open_workbook_auto_from_rs(cursor)
            .map_err(|e| AppError::Excel(format!("Failed to open Excel file: {}", e)))?;

        // Get the first sheet
        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = sheet_names
            .first()
            .ok_or_else(|| AppError::Excel("No sheets found in Excel file".to_string()))?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| AppError::Excel(format!("Failed to read sheet: {}", e)))?;

        let mut hotels = Vec::new();
        let mut header_row: Option<HeaderMapping> = None;

        for (row_idx, row) in range.rows().enumerate() {
            if row_idx == 0 {
                // Parse header row
                header_row = Some(parse_header(row)?);
                continue;
            }

            let headers = header_row
                .as_ref()
                .ok_or_else(|| AppError::Excel("Missing header row".to_string()))?;

            // Extract hotel data
            let hotel_name = get_cell_string(row, headers.hotel_name_col)?;
            let city = get_cell_string(row, headers.city_col)?;
            let country = get_cell_string(row, headers.country_col)
                .unwrap_or_else(|_| "Thailand".to_string());

            // Skip empty rows
            if hotel_name.trim().is_empty() {
                continue;
            }

            hotels.push(HotelImportData {
                hotel_name,
                city,
                country,
            });
        }

        if hotels.is_empty() {
            return Err(AppError::Excel("No hotels found in Excel file".to_string()));
        }

        Ok(hotels)
    }
}

/// Header column mapping
struct HeaderMapping {
    hotel_name_col: usize,
    city_col: usize,
    country_col: usize,
}

/// Parse header row to find column indices
fn parse_header(row: &[Data]) -> Result<HeaderMapping, AppError> {
    let mut hotel_name_col: Option<usize> = None;
    let mut city_col: Option<usize> = None;
    let mut country_col: Option<usize> = None;

    for (idx, cell) in row.iter().enumerate() {
        let value = format!("{cell}").to_lowercase();

        if value.contains("hotel") && value.contains("name") || value == "hotel" {
            hotel_name_col = Some(idx);
        } else if value.contains("city") || value.contains("location") {
            city_col = Some(idx);
        } else if value.contains("country") {
            country_col = Some(idx);
        }
    }

    // Fallback: use first three columns if headers not found
    let hotel_name_col = hotel_name_col.unwrap_or(0);
    let city_col = city_col.unwrap_or(1);
    let country_col = country_col.unwrap_or(2);

    Ok(HeaderMapping {
        hotel_name_col,
        city_col,
        country_col,
    })
}

/// Get string value from cell
fn get_cell_string(row: &[Data], col: usize) -> Result<String, AppError> {
    row.get(col)
        .map(|cell| format!("{cell}").trim().to_string())
        .ok_or_else(|| AppError::Excel(format!("Missing value in column {}", col)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_parsing() {
        let row = vec![
            Data::String("Hotel Name".to_string()),
            Data::String("City".to_string()),
            Data::String("Country".to_string()),
        ];

        let mapping = parse_header(&row).unwrap();
        assert_eq!(mapping.hotel_name_col, 0);
        assert_eq!(mapping.city_col, 1);
        assert_eq!(mapping.country_col, 2);
    }
}
