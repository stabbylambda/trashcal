use anyhow::Result;

use crate::trashcal::trashcal;
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, EXPIRES};
use http::StatusCode;
use lambda_http::{Body, Request, RequestExt, Response};
use lambda_runtime::tracing::info;

use tracing::instrument;

pub mod error;
pub mod pickup;
pub mod pickup_calendar;
pub mod trashcal;

#[instrument]
pub async fn get_trashcal(id: &str, whimsy: bool) -> Result<Response<Body>> {
    // Format is chosen by the URL suffix alone, never the Accept header: CloudFront's
    // cache key always includes the path, so `.json` and the default (iCal) land in
    // separate cache entries instead of colliding behind one URL.
    let is_json_request = id.contains(".json");

    let calendar = trashcal(id).await?;

    // build the response as either json or calendar
    let resp = Response::builder().status(StatusCode::OK);
    let resp = if is_json_request {
        info!(
            message = "Returning calendar as JSON", 
            address = %calendar.address,
            pickup_dates = ?calendar.pickups.iter().map(|p| p.date).collect::<Vec<_>>()
        );
        let json = serde_json::to_string_pretty(&calendar)?;

        resp.header(CONTENT_TYPE, "application/json")
            .header(EXPIRES, calendar.expires_header())
            .body(json.into())
    } else {
        info!(
            message = "Returning calendar as iCal",
            address = %calendar.address,
            pickup_dates = ?calendar.pickups.iter().map(|p| p.date).collect::<Vec<_>>()
        );
        let expires = calendar.expires_header();
        let calendar = calendar.to_calendar(whimsy)?;

        resp.header(CONTENT_TYPE, "text/calendar;charset=UTF-8")
            .header(CONTENT_DISPOSITION, "attachment; filename=trashcal.ics")
            .header(EXPIRES, expires)
            .body(calendar.to_string().into())
    };
    Ok(resp?)
}

pub async fn trashcal_handler(event: Request) -> Result<Response<Body>> {
    // get the ID
    let params = event.path_parameters();
    let query = event.query_string_parameters();
    let id = params
        .first("id")
        .or_else(|| query.first("id"))
        .unwrap_or("null");

    // get the whimsy parameter, defaults to true
    let whimsy = query
        .first("whimsy")
        .map(|v| v != "false")
        .unwrap_or(true);

    get_trashcal(id, whimsy).await
}
