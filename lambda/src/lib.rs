use anyhow::Result;
use http::{HeaderMap, HeaderValue};

use crate::trashcal::trashcal;
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use http::StatusCode;
use icalendar::Calendar;
use lambda_http::{Body, Request, RequestExt, Response};
use tracing::info;
use tracing::{span, Level};

pub mod error;
pub mod pickup;
pub mod pickup_calendar;
pub mod trashcal;

pub async fn trashcal_handler(event: Request) -> Result<Response<Body>> {
    // get the ID
    let params = event.path_parameters();
    let query = event.query_string_parameters();
    let id = params
        .first("id")
        .or_else(|| query.first("id"))
        .unwrap_or("null");

    // start tracing
    let _x = span!(Level::INFO, "trashcal", id).entered();

    let calendar = trashcal(id).await?;
    let accept = get_mime_type(event.headers());

    // build the response as either json or calendar
    let resp = Response::builder().status(StatusCode::OK);
    let resp = if accept.starts_with("application/json") {
        info!("Returning calendar as json");
        let json = serde_json::to_string_pretty(&calendar)?;

        resp.header(CONTENT_TYPE, "application/json")
            .body(json.into())
    } else {
        info!("Returning calendar as iCal feed");
        let calendar = Calendar::try_from(calendar)?;

        resp.header(CONTENT_TYPE, "text/calendar;charset=UTF-8")
            .header(CONTENT_DISPOSITION, "attachment; filename=trashcal.ics")
            .body(calendar.to_string().into())
    };

    Ok(resp?)
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
