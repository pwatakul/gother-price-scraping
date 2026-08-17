//! Spreadsheet Reader
//!
//! Reads hotel data from uploaded spreadsheets — .xlsx/.xls/.ods via
//! calamine, and .csv directly. The real 2,200-hotel master list ships as
//! a CSV (`docs/data/hotel-list-2200.csv`), so CSV is a first-class input
//! rather than a convenience.
//!
//! Both formats are normalized to the same `Vec<Vec<Data>>` shape, so all
//! the header-matching and row-extraction logic below is format-agnostic.

use calamine::{open_workbook_auto_from_rs, Data, Reader};
use std::io::Cursor;

use crate::error::AppError;
use crate::models::{HotelImportData, JobHotelParamOverride, MasterHotelImportRow};


/// Rows of cells from an uploaded file, whatever its format.
///
/// Format is detected from the leading bytes rather than the filename:
/// xlsx/ods are ZIP archives (`PK\x03\x04`) and legacy .xls is an OLE2
/// compound file (`\xD0\xCF\x11\xE0`). Anything else is treated as
/// delimited text — a filename can lie or be missing, the magic bytes
/// cannot.
fn read_rows(data: &[u8]) -> Result<Vec<Vec<Data>>, AppError> {
    const ZIP_MAGIC: &[u8] = b"PK\x03\x04";
    const OLE2_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0];

    let is_spreadsheet = data.starts_with(ZIP_MAGIC) || data.starts_with(OLE2_MAGIC);

    if is_spreadsheet {
        let cursor = Cursor::new(data);
        let mut workbook = open_workbook_auto_from_rs(cursor)
            .map_err(|e| AppError::Excel(format!("Failed to open spreadsheet: {}", e)))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = sheet_names
            .first()
            .ok_or_else(|| AppError::Excel("No sheets found in file".to_string()))?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| AppError::Excel(format!("Failed to read sheet: {}", e)))?;

        return Ok(range.rows().map(|row| row.to_vec()).collect());
    }

    read_csv_rows(data)
}

/// Parse CSV into the same cell shape as a worksheet. Every field becomes
/// a `Data::String`; the existing extractors format cells to strings and
/// parse numbers themselves, so nothing downstream needs to care.
fn read_csv_rows(data: &[u8]) -> Result<Vec<Vec<Data>>, AppError> {
    // Excel writes a UTF-8 BOM on CSV export; left in place it becomes
    // part of the first header cell and that column stops matching.
    let data = data.strip_prefix(b"\xef\xbb\xbf").unwrap_or(data);

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false) // the caller treats row 0 as the header
        .flexible(true) // ragged rows are the norm in exported lists
        .from_reader(data);

    let mut rows = Vec::new();
    for record in reader.records() {
        let record =
            record.map_err(|e| AppError::Excel(format!("Failed to read CSV: {}", e)))?;
        rows.push(
            record
                .iter()
                .map(|field| Data::String(field.trim().to_string()))
                .collect(),
        );
    }

    if rows.is_empty() {
        return Err(AppError::Excel("CSV file is empty".to_string()));
    }

    Ok(rows)
}

/// Spreadsheet reader — Excel or CSV.
pub struct ExcelReader;

impl ExcelReader {
    /// Read hotels from Excel file bytes
    pub fn read_hotels(data: &[u8]) -> Result<Vec<HotelImportData>, AppError> {
        let rows = read_rows(data)?;

        let mut hotels = Vec::new();
        let mut header_row: Option<HeaderMapping> = None;

        for (row_idx, row) in rows.iter().enumerate() {
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

    /// Read the real master hotel-list format (REQ-001-v1.2 F-021):
    /// `No, HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country, SEARCH`.
    /// `No` and `SEARCH` are ignored. Hyphenated hotel names are converted to
    /// spaces (the source list uses e.g. "Grand-Hyatt-Bangkok").
    pub fn read_master_hotel_list(data: &[u8]) -> Result<Vec<MasterHotelImportRow>, AppError> {
        let rows = read_rows(data)?;

        let mut rows_out = Vec::new();
        let mut header: Option<MasterHeaderMapping> = None;

        for (row_idx, row) in rows.iter().enumerate() {
            if row_idx == 0 {
                header = Some(parse_master_header(row)?);
                continue;
            }

            let headers = header
                .as_ref()
                .ok_or_else(|| AppError::Excel("Missing header row".to_string()))?;

            let hid_str = get_cell_string(row, headers.hid_col).unwrap_or_default();
            if hid_str.trim().is_empty() {
                continue;
            }
            let Ok(hid) = hid_str.trim().parse::<i64>() else {
                continue;
            };

            let hotel_name = get_cell_string(row, headers.hotel_name_col)
                .unwrap_or_default()
                .replace('-', " ");
            if hotel_name.trim().is_empty() {
                continue;
            }

            rows_out.push(MasterHotelImportRow {
                hid,
                hotel_name,
                update_url: headers
                    .update_url_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .filter(|s| !s.trim().is_empty()),
                slug: headers
                    .slug_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .filter(|s| !s.trim().is_empty()),
                supplier_type: headers
                    .supplier_type_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .filter(|s| !s.trim().is_empty()),
                country: headers
                    .country_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .unwrap_or_else(|| "thailand".to_string()),
            });
        }

        if rows_out.is_empty() {
            return Err(AppError::Excel(
                "No hotels found in master hotel-list file".to_string(),
            ));
        }

        Ok(rows_out)
    }

    /// Read an optional per-job search-parameter override sheet: hotel
    /// identified by `hid` or `hotel_name`, plus optional checkin_date /
    /// checkout_date / rooms / adults / currency. Blank cells map to `None`
    /// so the caller can fall back to job-level defaults.
    pub fn read_job_hotel_overrides(
        data: &[u8],
    ) -> Result<Vec<JobHotelParamOverride>, AppError> {
        let rows = read_rows(data)?;

        let mut overrides = Vec::new();
        let mut header: Option<OverrideHeaderMapping> = None;

        for (row_idx, row) in rows.iter().enumerate() {
            if row_idx == 0 {
                header = Some(parse_override_header(row)?);
                continue;
            }

            let headers = header
                .as_ref()
                .ok_or_else(|| AppError::Excel("Missing header row".to_string()))?;

            let hid = headers
                .hid_col
                .and_then(|c| get_cell_string(row, c).ok())
                .and_then(|s| s.trim().parse::<i64>().ok());
            let hotel_name = headers
                .hotel_name_col
                .and_then(|c| get_cell_string(row, c).ok())
                .filter(|s| !s.trim().is_empty());

            if hid.is_none() && hotel_name.is_none() {
                continue;
            }

            overrides.push(JobHotelParamOverride {
                hid,
                hotel_name,
                checkin_date: headers
                    .checkin_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .and_then(|s| chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()),
                checkout_date: headers
                    .checkout_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .and_then(|s| chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()),
                rooms: headers
                    .rooms_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .and_then(|s| s.trim().parse::<i32>().ok()),
                adults: headers
                    .adults_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .and_then(|s| s.trim().parse::<i32>().ok()),
                currency: headers
                    .currency_col
                    .and_then(|c| get_cell_string(row, c).ok())
                    .filter(|s| !s.trim().is_empty()),
            });
        }

        Ok(overrides)
    }
}

struct MasterHeaderMapping {
    hid_col: usize,
    hotel_name_col: usize,
    update_url_col: Option<usize>,
    slug_col: Option<usize>,
    supplier_type_col: Option<usize>,
    country_col: Option<usize>,
}

fn parse_master_header(row: &[Data]) -> Result<MasterHeaderMapping, AppError> {
    let mut hid_col = None;
    let mut hotel_name_col = None;
    let mut update_url_col = None;
    let mut slug_col = None;
    let mut supplier_type_col = None;
    let mut country_col = None;

    for (idx, cell) in row.iter().enumerate() {
        let value = format!("{cell}").to_lowercase();

        if value == "hid" {
            hid_col = Some(idx);
        } else if value.contains("hotel") && value.contains("name") {
            hotel_name_col = Some(idx);
        } else if value.contains("url") {
            update_url_col = Some(idx);
        } else if value.contains("slug") {
            slug_col = Some(idx);
        } else if value.contains("supplier") {
            supplier_type_col = Some(idx);
        } else if value.contains("country") {
            country_col = Some(idx);
        }
    }

    let hid_col = hid_col.ok_or_else(|| AppError::Excel("Missing HID column".to_string()))?;
    let hotel_name_col = hotel_name_col
        .ok_or_else(|| AppError::Excel("Missing Hotel-Name column".to_string()))?;

    Ok(MasterHeaderMapping {
        hid_col,
        hotel_name_col,
        update_url_col,
        slug_col,
        supplier_type_col,
        country_col,
    })
}

struct OverrideHeaderMapping {
    hid_col: Option<usize>,
    hotel_name_col: Option<usize>,
    checkin_col: Option<usize>,
    checkout_col: Option<usize>,
    rooms_col: Option<usize>,
    adults_col: Option<usize>,
    currency_col: Option<usize>,
}

fn parse_override_header(row: &[Data]) -> Result<OverrideHeaderMapping, AppError> {
    let mut hid_col = None;
    let mut hotel_name_col = None;
    let mut checkin_col = None;
    let mut checkout_col = None;
    let mut rooms_col = None;
    let mut adults_col = None;
    let mut currency_col = None;

    for (idx, cell) in row.iter().enumerate() {
        let value = format!("{cell}").to_lowercase();

        if value == "hid" {
            hid_col = Some(idx);
        } else if value.contains("hotel") && value.contains("name") {
            hotel_name_col = Some(idx);
        } else if value.contains("checkin") || value.contains("check_in") || value.contains("check-in")
        {
            checkin_col = Some(idx);
        } else if value.contains("checkout") || value.contains("check_out") || value.contains("check-out")
        {
            checkout_col = Some(idx);
        } else if value.contains("room") {
            rooms_col = Some(idx);
        } else if value.contains("adult") {
            adults_col = Some(idx);
        } else if value.contains("currency") {
            currency_col = Some(idx);
        }
    }

    Ok(OverrideHeaderMapping {
        hid_col,
        hotel_name_col,
        checkin_col,
        checkout_col,
        rooms_col,
        adults_col,
        currency_col,
    })
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

    #[test]
    fn test_master_header_parsing() {
        // Real hotel-list-2200.csv shape.
        let row = vec![
            Data::String("No".to_string()),
            Data::String("HID".to_string()),
            Data::String("Hotel-Name".to_string()),
            Data::String("UPDATE URL 15 JAN".to_string()),
            Data::String("SLUG".to_string()),
            Data::String("Supplier-or-Direct".to_string()),
            Data::String("Country".to_string()),
            Data::String("SEARCH".to_string()),
        ];

        let mapping = parse_master_header(&row).unwrap();
        assert_eq!(mapping.hid_col, 1);
        assert_eq!(mapping.hotel_name_col, 2);
        assert_eq!(mapping.update_url_col, Some(3));
        assert_eq!(mapping.slug_col, Some(4));
        assert_eq!(mapping.supplier_type_col, Some(5));
        assert_eq!(mapping.country_col, Some(6));
    }
}

#[cfg(test)]
mod csv_tests {
    use super::*;

    /// The real master list ships as CSV with this exact header
    /// (docs/data/hotel-list-2200.csv), so it must import as-is.
    const MASTER_CSV: &[u8] = b"No,HID,Hotel-Name,UPDATE URL 15 JAN,SLUG,Supplier-or-Direct,Country,SEARCH\n\
190,1022,Asia-Airport-Donmuang-Hotel,www.gother.com/th-th/hotels/x,thailand/pathum-thani/asia-airport,DIRECT,thailand,#N/A\n\
364,772485,Arden-Hotel-&-Residence-Pattaya,www.gother.com/th-th/hotels/y,thailand/chonburi/arden,DIRECT,thailand,#N/A\n";

    #[test]
    fn reads_the_real_master_csv() {
        let rows = ExcelReader::read_master_hotel_list(MASTER_CSV).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].hid, 1022);
        // Hyphenated source names are converted to spaces.
        assert_eq!(rows[0].hotel_name, "Asia Airport Donmuang Hotel");
        assert_eq!(rows[1].hid, 772485);
    }

    /// Excel writes a UTF-8 BOM on CSV export; left in place it becomes
    /// part of the first header cell and that column stops matching.
    #[test]
    fn tolerates_a_utf8_bom() {
        let mut with_bom = vec![0xEF, 0xBB, 0xBF];
        with_bom.extend_from_slice(MASTER_CSV);
        let rows = ExcelReader::read_master_hotel_list(&with_bom).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].hid, 1022, "BOM must not break the HID column");
    }

    #[test]
    fn reads_the_plain_three_column_csv() {
        let csv = b"hotel_name,city,country\nMandarin Oriental,Bangkok,Thailand\nRayavadee,Krabi,Thailand\n";
        let hotels = ExcelReader::read_hotels(csv).unwrap();
        assert_eq!(hotels.len(), 2);
        assert_eq!(hotels[0].hotel_name, "Mandarin Oriental");
        assert_eq!(hotels[1].city, "Krabi");
    }

    /// Quoted fields containing commas must not split — hotel names in the
    /// real list include them.
    #[test]
    fn respects_quoted_fields() {
        let csv = b"hotel_name,city,country\n\"Sofitel, Sukhumvit\",Bangkok,Thailand\n";
        let hotels = ExcelReader::read_hotels(csv).unwrap();
        assert_eq!(hotels[0].hotel_name, "Sofitel, Sukhumvit");
    }

    #[test]
    fn empty_csv_is_an_error_not_a_panic() {
        assert!(ExcelReader::read_hotels(b"").is_err());
    }

    /// A ZIP magic number must still route to calamine, not the CSV path.
    #[test]
    fn detects_xlsx_by_magic_bytes_not_extension() {
        let fake_xlsx = b"PK\x03\x04not-really-a-zip";
        let err = read_rows(fake_xlsx).unwrap_err();
        // Reached calamine and failed there, rather than being parsed as CSV.
        assert!(
            format!("{err:?}").contains("spreadsheet"),
            "should route to the spreadsheet reader, got: {err:?}"
        );
    }
}
