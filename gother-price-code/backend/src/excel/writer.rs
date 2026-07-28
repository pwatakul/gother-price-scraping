//! Excel Writer
//!
//! Writes scrape results and templates to Excel files.

use rust_xlsxwriter::{Color, Format, Workbook, Worksheet};

use crate::error::AppError;
use crate::models::ScrapeResultsResponse;

/// Excel file writer
pub struct ExcelWriter;

impl ExcelWriter {
    /// Create hotel import template
    pub fn create_import_template() -> Result<Vec<u8>, AppError> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Header format
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        // Set column widths
        worksheet.set_column_width(0, 40).map_err(xlsx_err)?;
        worksheet.set_column_width(1, 20).map_err(xlsx_err)?;
        worksheet.set_column_width(2, 20).map_err(xlsx_err)?;

        // Write headers
        worksheet
            .write_string_with_format(0, 0, "hotel_name", &header_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_string_with_format(0, 1, "city", &header_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_string_with_format(0, 2, "country", &header_format)
            .map_err(xlsx_err)?;

        // Write example rows
        worksheet
            .write_string(1, 0, "Dusit Thani Bangkok")
            .map_err(xlsx_err)?;
        worksheet.write_string(1, 1, "Bangkok").map_err(xlsx_err)?;
        worksheet.write_string(1, 2, "Thailand").map_err(xlsx_err)?;

        worksheet
            .write_string(2, 0, "Mandarin Oriental")
            .map_err(xlsx_err)?;
        worksheet.write_string(2, 1, "Bangkok").map_err(xlsx_err)?;
        worksheet.write_string(2, 2, "Thailand").map_err(xlsx_err)?;

        // Save to buffer
        let buffer = workbook.save_to_buffer().map_err(xlsx_err)?;
        Ok(buffer)
    }

    /// Write scrape results to Excel
    pub fn write_results(results: &ScrapeResultsResponse) -> Result<Vec<u8>, AppError> {
        let mut workbook = Workbook::new();

        // Summary sheet
        Self::write_summary_sheet(&mut workbook, results)?;

        // Detailed results sheet
        Self::write_results_sheet(&mut workbook, results)?;

        // Save to buffer
        let buffer = workbook.save_to_buffer().map_err(xlsx_err)?;
        Ok(buffer)
    }

    /// Write summary sheet
    fn write_summary_sheet(
        workbook: &mut Workbook,
        results: &ScrapeResultsResponse,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Summary").map_err(xlsx_err)?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        let label_format = Format::new().set_bold();

        // Title
        worksheet
            .write_string_with_format(0, 0, "Hotel Price Comparison Report", &header_format)
            .map_err(xlsx_err)?;
        worksheet
            .merge_range(0, 0, 0, 3, "Hotel Price Comparison Report", &header_format)
            .map_err(xlsx_err)?;

        // Job info
        worksheet
            .write_string_with_format(2, 0, "Check-in Date:", &label_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_string(2, 1, &results.job.checkin_date)
            .map_err(xlsx_err)?;

        worksheet
            .write_string_with_format(3, 0, "Check-out Date:", &label_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_string(3, 1, &results.job.checkout_date)
            .map_err(xlsx_err)?;

        worksheet
            .write_string_with_format(4, 0, "Rooms:", &label_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_number(4, 1, results.job.rooms as f64)
            .map_err(xlsx_err)?;

        worksheet
            .write_string_with_format(5, 0, "Adults:", &label_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_number(5, 1, results.job.adults as f64)
            .map_err(xlsx_err)?;

        // Summary stats
        worksheet
            .write_string_with_format(7, 0, "Summary", &header_format)
            .map_err(xlsx_err)?;
        worksheet
            .merge_range(7, 0, 7, 1, "Summary", &header_format)
            .map_err(xlsx_err)?;

        worksheet
            .write_string_with_format(8, 0, "Total Hotels:", &label_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_number(8, 1, results.summary.total_hotels as f64)
            .map_err(xlsx_err)?;

        worksheet
            .write_string_with_format(9, 0, "Successful:", &label_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_number(9, 1, results.summary.successful as f64)
            .map_err(xlsx_err)?;

        worksheet
            .write_string_with_format(10, 0, "Failed:", &label_format)
            .map_err(xlsx_err)?;
        worksheet
            .write_number(10, 1, results.summary.failed as f64)
            .map_err(xlsx_err)?;

        if let Some(avg) = results.summary.avg_best_price {
            worksheet
                .write_string_with_format(11, 0, "Avg Best Price:", &label_format)
                .map_err(xlsx_err)?;
            worksheet.write_number(11, 1, avg).map_err(xlsx_err)?;
        }

        // Set column widths
        worksheet.set_column_width(0, 20).map_err(xlsx_err)?;
        worksheet.set_column_width(1, 25).map_err(xlsx_err)?;

        Ok(())
    }

    /// Write detailed results sheet
    fn write_results_sheet(
        workbook: &mut Workbook,
        results: &ScrapeResultsResponse,
    ) -> Result<(), AppError> {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Results").map_err(xlsx_err)?;

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        let best_price_format = Format::new()
            .set_background_color(Color::RGB(0xC6EFCE))
            .set_font_color(Color::RGB(0x006100));

        let failed_format = Format::new()
            .set_background_color(Color::RGB(0xFFC7CE))
            .set_font_color(Color::RGB(0x9C0006));

        // Headers
        let headers = [
            "Hotel Name",
            "City",
            "Country",
            "Status",
            "Gother (THB)",
            "Agoda (THB)",
            "Booking (THB)",
            "Trip.com (THB)",
            "Official (THB)",
            "Best Source",
            "Best Price (THB)",
            "Diff vs Gother",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(xlsx_err)?;
        }

        // Data rows
        for (row_idx, hotel) in results.results.iter().enumerate() {
            let row = (row_idx + 1) as u32;

            worksheet
                .write_string(row, 0, &hotel.hotel.name)
                .map_err(xlsx_err)?;
            worksheet
                .write_string(row, 1, &hotel.hotel.city)
                .map_err(xlsx_err)?;
            worksheet
                .write_string(row, 2, &hotel.hotel.country)
                .map_err(xlsx_err)?;
            worksheet
                .write_string(row, 3, &hotel.status.to_string())
                .map_err(xlsx_err)?;

            if hotel.status == crate::models::HotelScrapeStatus::Failed {
                // Write error message
                if let Some(err) = &hotel.error_message {
                    worksheet
                        .write_string_with_format(row, 4, err, &failed_format)
                        .map_err(xlsx_err)?;
                }
                continue;
            }

            // Find prices by source
            let gother_price = hotel
                .prices
                .iter()
                .find(|p| p.source == "gother")
                .map(|p| p.price_thb);
            let agoda_price = hotel
                .prices
                .iter()
                .find(|p| p.source == "agoda")
                .map(|p| p.price_thb);
            let booking_price = hotel
                .prices
                .iter()
                .find(|p| p.source == "booking")
                .map(|p| p.price_thb);
            let trip_price = hotel
                .prices
                .iter()
                .find(|p| p.source == "trip.com")
                .map(|p| p.price_thb);
            let official_price = hotel
                .prices
                .iter()
                .find(|p| p.source == "official")
                .map(|p| p.price_thb);

            // Write prices
            if let Some(price) = gother_price {
                worksheet.write_number(row, 4, price).map_err(xlsx_err)?;
            }
            if let Some(price) = agoda_price {
                let is_best = hotel.best_source.as_deref() == Some("agoda");
                if is_best {
                    worksheet
                        .write_number_with_format(row, 5, price, &best_price_format)
                        .map_err(xlsx_err)?;
                } else {
                    worksheet.write_number(row, 5, price).map_err(xlsx_err)?;
                }
            }
            if let Some(price) = booking_price {
                let is_best = hotel.best_source.as_deref() == Some("booking");
                if is_best {
                    worksheet
                        .write_number_with_format(row, 6, price, &best_price_format)
                        .map_err(xlsx_err)?;
                } else {
                    worksheet.write_number(row, 6, price).map_err(xlsx_err)?;
                }
            }
            if let Some(price) = trip_price {
                let is_best = hotel.best_source.as_deref() == Some("trip.com");
                if is_best {
                    worksheet
                        .write_number_with_format(row, 7, price, &best_price_format)
                        .map_err(xlsx_err)?;
                } else {
                    worksheet.write_number(row, 7, price).map_err(xlsx_err)?;
                }
            }
            if let Some(price) = official_price {
                let is_best = hotel.best_source.as_deref() == Some("official");
                if is_best {
                    worksheet
                        .write_number_with_format(row, 8, price, &best_price_format)
                        .map_err(xlsx_err)?;
                } else {
                    worksheet.write_number(row, 8, price).map_err(xlsx_err)?;
                }
            }

            // Best source and price
            if let Some(source) = &hotel.best_source {
                worksheet.write_string(row, 9, source).map_err(xlsx_err)?;
            }
            if let Some(price) = hotel.best_price {
                worksheet
                    .write_number_with_format(row, 10, price, &best_price_format)
                    .map_err(xlsx_err)?;
            }
            if let Some(diff) = hotel.price_difference {
                worksheet.write_number(row, 11, diff).map_err(xlsx_err)?;
            }
        }

        // Set column widths
        let widths = [30, 15, 15, 10, 15, 15, 15, 15, 15, 12, 15, 15];
        for (col, width) in widths.iter().enumerate() {
            worksheet
                .set_column_width(col as u16, *width)
                .map_err(xlsx_err)?;
        }

        Ok(())
    }
}

/// Convert xlsx error to AppError
fn xlsx_err<E: std::fmt::Display>(e: E) -> AppError {
    AppError::Excel(format!("Excel write error: {}", e))
}
