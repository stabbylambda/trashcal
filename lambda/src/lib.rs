use anyhow::Result;
use http::{HeaderMap, HeaderValue};

use crate::trashcal::trashcal;
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, EXPIRES};
use http::StatusCode;
use icalendar::Calendar;
use lambda_http::{Body, Request, RequestExt, Response};
use lambda_runtime::tracing::info;

use tracing::instrument;

pub mod error;
pub mod pickup;
pub mod pickup_calendar;
pub mod trashcal;

#[instrument]
pub async fn get_trashcal(id: &str, accept: &str) -> Result<Response<Body>> {
    let is_ics_request = id.contains(".ics");

    let calendar = trashcal(id).await?;

    // build the response as either json or calendar
    let resp = Response::builder().status(StatusCode::OK);
    let resp = if accept.starts_with("application/json") && !is_ics_request {
        info!(message = "Returning calendar as JSON");
        let json = serde_json::to_string_pretty(&calendar)?;

        resp.header(CONTENT_TYPE, "application/json")
            .header(EXPIRES, calendar.expires_header())
            .body(json.into())
    } else {
        info!(message = "Returning calendar as iCal");
        let expires = calendar.expires_header();
        let calendar = Calendar::try_from(calendar)?;

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
    let accept = get_mime_type(event.headers());

    get_trashcal(id, accept).await
}

/// Safely get the accept header. Fastmail apparently doesn't send an accept header at all (!)
pub fn get_mime_type(headers: &HeaderMap<HeaderValue>) -> &str {
    headers
        .get(http::header::ACCEPT)
        .and_then(|x| x.to_str().ok())
        .unwrap_or("text/calendar")
}

#[cfg(test)]
mod test {
    use http::HeaderMap;

    use crate::get_mime_type;

    #[test]
    fn test_missing_accept_header() {
        let headers = HeaderMap::new();
        let result = get_mime_type(&headers);
        assert_eq!(result, "text/calendar");
    }

    #[test]
    fn test_json() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::ACCEPT, "application/json".parse().unwrap());
        let result = get_mime_type(&headers);
        assert_eq!(result, "application/json");
    }

    #[test]
    fn test_calendar() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::ACCEPT, "text/calendar".parse().unwrap());
        let result = get_mime_type(&headers);
        assert_eq!(result, "text/calendar");
    }

    #[test]
    fn test_weird_header_value() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::ACCEPT, "💩".parse().unwrap());
        let result = get_mime_type(&headers);
        assert_eq!(result, "text/calendar");
    }
}
