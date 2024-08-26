use std::sync::LazyLock;

use crate::error::Error;
use chrono::NaiveDate;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(
    EnumString, Display, Serialize, Deserialize, Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy,
)]
pub enum PickupType {
    #[serde(rename(serialize = "♻️ Recyclables", deserialize = "♻️ Recyclables"))]
    #[strum(to_string = "♻️ Recyclables", serialize = "Recyclables")]
    Recyclables,

    #[serde(rename(serialize = "🌳 Organics", deserialize = "🌳 Organics"))]
    #[strum(to_string = "🌳 Organics", serialize = "Organics")]
    Organics,

    #[serde(rename(serialize = "🗑️ Trash", deserialize = "🗑️ Trash"))]
    #[strum(to_string = "🗑️ Trash", serialize = "Trash")]
    Trash,
}

pub(crate) fn nth_text<'a>(
    x: ElementRef<'a>,
    selector: &'a Selector,
    n: usize,
) -> Result<&'a str, Error> {
    x.select(selector)
        .nth(n)
        .and_then(|e| e.text().next())
        .map(|t| t.trim())
        .ok_or(Error::ParseError)
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Serialize, Deserialize)]
pub struct Pickup {
    pub date: NaiveDate,
    pub name: PickupType,
}

impl Pickup {
    pub(crate) fn new(name: PickupType, date: NaiveDate) -> Self {
        Pickup { date, name }
    }
}

impl TryFrom<&str> for Pickup {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let html = Html::parse_fragment(value);
        Pickup::try_from(html.root_element())
    }
}

static NAME_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h3").unwrap());
static DATE_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("p").unwrap());

impl TryFrom<ElementRef<'_>> for Pickup {
    type Error = Error;

    fn try_from(value: ElementRef) -> Result<Self, Self::Error> {
        let name = nth_text(value, &NAME_SELECTOR, 0)?;
        let date = nth_text(value, &DATE_SELECTOR, 2)?;
        let name: PickupType = name.parse()?;
        let date = NaiveDate::parse_from_str(date, "%m/%d/%Y")?;

        Ok(Pickup::new(name, date))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::Pickup;
    use super::PickupType;

    fn create_pickup_html(name: &str, date: &str) -> String {
        format!("<div><h3>{name}</h3><p></p><p></p><p>{date}</p></div>")
    }

    #[test]
    fn parse_single_pickup() {
        let html = create_pickup_html("Trash", "01/01/2023");
        let result = Pickup::try_from(html.as_str()).unwrap();

        assert_eq!(
            result,
            Pickup {
                name: PickupType::Trash,
                date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            }
        )
    }

    #[test]
    fn parse_single_pickup_with_bad_name() {
        let html = create_pickup_html("Not a real thing", "01/01/2023");
        let result = Pickup::try_from(html.as_str());

        assert!(result.is_err());
    }
}
