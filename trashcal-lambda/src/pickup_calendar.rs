use crate::error::Error;
use crate::pickup::nth_text;
use crate::pickup::{Pickup, PickupType};
use icalendar::{Calendar, Component, Event, EventLike};
use itertools::Itertools;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static ADDRESS_SELECTOR: OnceLock<Selector> = OnceLock::new();
static SCHEDULE_SELECTOR: OnceLock<Selector> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct PickupCalendar {
    pub id: String,
    pub address: String,
    pub pickups: Vec<Pickup>,
}

impl PickupCalendar {
    fn new(id: &str, address: &str, pickups: Vec<Pickup>) -> PickupCalendar {
        let dates = pickups
            .into_iter()
            .sorted()
            .chunk_by(|e| e.date)
            .into_iter()
            .flat_map(|(date, pickups)| {
                let mut pickups = pickups.collect_vec();

                // For any date without an existing trash and organics item, we need to add them, since they're every week
                if pickups.len() == 1 {
                    pickups.append(&mut vec![
                        Pickup::new(PickupType::Trash, date),
                        Pickup::new(PickupType::Organics, date),
                    ]);
                }

                pickups.sort_by(|x, y| x.name.cmp(&y.name));
                pickups
            })
            .collect_vec();

        PickupCalendar {
            id: id.to_string(),
            address: address.to_string(),
            pickups: dates,
        }
    }
}

impl TryFrom<PickupCalendar> for Calendar {
    type Error = Error;

    fn try_from(value: PickupCalendar) -> Result<Self, Self::Error> {
        let url = format!("https://stabbylambda.com/trashcal/{}", value.id);
        let description = format!(
            "Trashcal: {url}

SD Trash Page: https://getitdone.sandiego.gov/CollectionDetail?id={}",
            value.id
        );

        // Create new calendar events and add them
        let events = value.pickups.into_iter().map(|pickup| {
            Event::new()
                .all_day(pickup.date)
                .url(&url)
                .summary(&pickup.name.to_string())
                .description(&description)
                .done()
        });

        let mut calendar = Calendar::new().name("Trashcal").done();
        calendar.extend(events);
        Ok(calendar)
    }
}

impl<'a> TryFrom<(&'a str, &'a Html)> for PickupCalendar {
    type Error = Error;

    fn try_from((id, document): (&'a str, &'a Html)) -> Result<Self, Self::Error> {
        let address_selector =
            ADDRESS_SELECTOR.get_or_init(|| Selector::parse("p.subheading").unwrap());
        let schedule_selector =
            SCHEDULE_SELECTOR.get_or_init(|| Selector::parse("div.schedule div").unwrap());

        let address = nth_text(document.root_element(), address_selector, 0)?;
        let schedule = document.select(schedule_selector);
        let pickups: Result<Vec<Pickup>, Error> = schedule
            .map(Pickup::try_from)
            .filter(Result::is_ok)
            .collect();

        Ok(PickupCalendar::new(id, address, pickups?))
    }
}

#[cfg(test)]
mod test {
    use chrono::Duration;
    use chrono::NaiveDate;
    use chrono::Utc;

    use super::Pickup;
    use super::PickupCalendar;
    use crate::pickup::PickupType;

    fn create_pickup_html(name: &str, date: &str) -> String {
        format!("<div><h3>{name}</h3><p></p><p></p><p>{date}</p></div>")
    }
    fn create_page_html(pairs: Vec<(&str, &str)>) -> String {
        let divs = pairs
            .iter()
            .map(|(name, date)| create_pickup_html(name, date))
            .collect::<Vec<String>>()
            .join("");

        format!("<html><body><div><p class=\"subheading\">1234 ANYWHERE ST, San Diego, CA 92101</p></div><div class=\"schedule\">{divs}</div></body></html>")
    }

    #[test]
    fn parse_multiple_pickups() {
        let html = create_page_html(vec![
            ("Trash", "01/01/2023"),
            ("Organics", "01/01/2023"),
            ("Recyclables", "01/01/2023"),
        ]);
        let document = scraper::Html::parse_document(&html);

        let expected = [
            Pickup::new(
                PickupType::Recyclables,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            ),
            Pickup::new(
                PickupType::Organics,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            ),
            Pickup::new(
                PickupType::Trash,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            ),
        ];

        let actual = PickupCalendar::try_from(("foo", &document)).unwrap();

        for (i, x) in expected.iter().enumerate() {
            let actual = actual.pickups.get(i).unwrap();
            assert_eq!(x, actual);
        }
    }

    #[test]
    fn insert_pickup_for_opposite_week_recycling() {
        let this_week = Utc::now().date_naive();
        let next_week = this_week.checked_add_signed(Duration::days(7)).unwrap();

        let pickups = vec![
            Pickup::new(PickupType::Trash, this_week),
            Pickup::new(PickupType::Recyclables, next_week),
            Pickup::new(PickupType::Organics, this_week),
        ];

        let result = PickupCalendar::new("foo", "1234 Anywhere St.", pickups);
        assert_eq!(
            result.pickups,
            vec![
                //this week
                Pickup::new(PickupType::Organics, this_week),
                Pickup::new(PickupType::Trash, this_week),
                // next week
                Pickup::new(PickupType::Recyclables, next_week),
                Pickup::new(PickupType::Organics, next_week),
                Pickup::new(PickupType::Trash, next_week),
            ]
        );
    }

    #[test]
    fn sameweek_greens_and_recycling() {
        let this_week = Utc::now().date_naive();

        let pickups = vec![
            Pickup::new(PickupType::Trash, this_week),
            Pickup::new(PickupType::Recyclables, this_week),
            Pickup::new(PickupType::Organics, this_week),
        ];

        let result = PickupCalendar::new("foo", "1234 Anywhere St.", pickups);
        assert_eq!(
            result.pickups,
            vec![
                Pickup::new(PickupType::Recyclables, this_week),
                Pickup::new(PickupType::Organics, this_week),
                Pickup::new(PickupType::Trash, this_week),
            ]
        );
    }
}
