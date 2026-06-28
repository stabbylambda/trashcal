use anyhow::{ensure, Result};
use lambda_runtime::tracing::info;
use scraper::Html;

use crate::{error::Error, pickup_calendar::PickupCalendar};

// Gets a trash calendar given an ID
pub async fn trashcal(id: &str) -> Result<PickupCalendar> {
    // as far as I can tell, all IDs start with a4O
    ensure!(id.starts_with("a4O"), Error::IdError(id.to_string()));

    // rip out the format suffix (.ics for Paul, .json for the website) before
    // building the upstream URL
    let id = id.replace(".ics", "").replace(".json", "");

    info!("Getting trashcal");
    let url = format!("https://getitdone.sandiego.gov/CollectionDetail?id={id}");
    let html = reqwest::get(url).await?.text().await?;

    // If we got the landing page, don't even try to parse it
    ensure!(
        !html.contains("handleRedirect"),
        Error::RedirectPage(id.to_string())
    );

    info!("Parsing calendar");
    let document = Html::parse_document(&html);
    let calendar = PickupCalendar::try_from((id.as_str(), &document))?;

    Ok(calendar)
}
