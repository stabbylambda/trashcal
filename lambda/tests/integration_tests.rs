use http::header::CONTENT_TYPE;
use trashcal::trashcal_handler;

#[tokio::test]
async fn path_based() {
    let input = include_str!("./data/path_based.json");
    let request = lambda_http::request::from_str(input).expect("failed to create request");
    let response = trashcal_handler(request).await.expect("Failed to execute");
    let body = std::str::from_utf8(response.body()).expect("Should have a body");
    assert_eq!(
        response.headers()[CONTENT_TYPE],
        "text/calendar;charset=UTF-8"
    );
    assert!(body.contains("X-WR-CALNAME:Trashcal"));
}

#[tokio::test]
async fn query_based() {
    let input = include_str!("./data/query_based.json");
    let request = lambda_http::request::from_str(input).expect("failed to create request");
    let response = trashcal_handler(request).await.expect("Failed to execute");
    let body = std::str::from_utf8(response.body()).expect("Should have a body");
    assert_eq!(
        response.headers()[CONTENT_TYPE],
        "text/calendar;charset=UTF-8"
    );
    assert!(body.contains("X-WR-CALNAME:Trashcal"));
}

#[tokio::test]
async fn path_based_with_json() {
    let input = include_str!("./data/path_based_with_json.json");
    let request = lambda_http::request::from_str(input).expect("failed to create request");
    let response = trashcal_handler(request).await.expect("Failed to execute");
    let body = std::str::from_utf8(response.body()).expect("Should have a body");
    assert_eq!(response.headers()[CONTENT_TYPE], "application/json");
    assert!(body.contains("1234 AGATE ST"));
}

#[tokio::test]
async fn fails_with_no_id() {
    let input = include_str!("./data/no_id.json");
    let request = lambda_http::request::from_str(input).expect("failed to create request");
    let response = trashcal_handler(request).await;
    assert!(response.is_err())
}

#[tokio::test]
async fn fails_with_bad_id() {
    let input = include_str!("./data/bad_id.json");
    let request = lambda_http::request::from_str(input).expect("failed to create request");
    let response = trashcal_handler(request).await;
    assert!(response.is_err())
}
