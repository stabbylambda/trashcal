use crate::error::Error;
use crate::pickup_calendar::PickupCalendar;
use scraper::Html;
use tracing::{error, info};

pub mod error;
pub mod pickup;
pub mod pickup_calendar;

// Gets a trash calendar given an ID
pub async fn trashcal(id: &str) -> Result<PickupCalendar, Error> {
    // as far as I can tell, all IDs start with a4Ot
    if !id.starts_with("a4Ot") {
        let err = Error::IdError(id.to_string());
        error!("{}", err);
        return Err(err);
    }

    info!("Getting trashcal");
    let url = format!("https://getitdone.force.com/CollectionDetail?id={id}");
    let html = reqwest::get(url).await?.text().await?;

    // If we got the landing page, don't even try to parse it
    if html.contains("handleRedirect") {
        let err = Error::RedirectPage(id.to_string());
        error!("{}", err);
        return Err(err);
    }

    info!("Parsing calendar");
    let document = Html::parse_document(&html);
    let calendar = PickupCalendar::try_from((id, &document))?;

    Ok(calendar)
}
