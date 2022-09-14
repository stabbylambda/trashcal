use crate::error::Error;
use crate::pickup_calendar::PickupCalendar;
use icalendar::Calendar;
use scraper::Html;
use tracing::info;

pub mod error;
pub mod pickup;
pub mod pickup_calendar;

// Gets a trash calendar given an ID
pub async fn trashcal(id: &str) -> Result<Calendar, Error> {
    // as far as I can tell, all IDs start with a4Ot
    if !id.starts_with("a4Ot") {
        return Err(Error::IdError(id.to_string()));
    }

    info!("Getting trashcal");
    let url = format!("https://getitdone.force.com/CollectionDetail?id={id}");
    let html = reqwest::get(url).await?.text().await?;

    // If we got the landing page, don't even try to parse it
    if html.contains("handleRedirect") {
        return Err(Error::RedirectPage(id.to_string()));
    }

    info!("Parsing calendar");
    let document = Html::parse_document(&html);
    let pickups = PickupCalendar::try_from((id, &document))?;

    info!("Generating iCal file");
    Calendar::try_from(pickups)
}
