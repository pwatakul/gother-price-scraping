//! Templates Handlers

use axum::{
    body::Body,
    http::{header, Response},
};

use crate::error::AppResult;
use crate::excel::ExcelWriter;

/// Download Excel template for hotel import
pub async fn download_template() -> AppResult<Response<Body>> {
    let template_data = ExcelWriter::create_import_template()?;

    let response = Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"hotel-import-template.xlsx\"",
        )
        .body(Body::from(template_data))
        .unwrap();

    Ok(response)
}
