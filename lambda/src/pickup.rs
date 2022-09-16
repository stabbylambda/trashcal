use crate::error::Error;
use chrono::NaiveDate;
use lazy_static::lazy_static;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use strum::{Display, EnumString};

lazy_static! {
    static ref NAME_SELECTOR: Selector = Selector::parse("h3").unwrap();
    static ref DATE_SELECTOR: Selector = Selector::parse("p").unwrap();
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, EnumString, Display, Clone, Copy)]
pub enum PickupType {
    #[strum(serialize = "Recyclables", to_string = "♻️ Recyclables")]
    Recyclables,

    #[strum(serialize = "Greens", to_string = "🌳 Greens")]
    Greens,

    #[strum(serialize = "Trash", to_string = "🗑️ Trash")]
    Trash,
}

impl Serialize for PickupType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            PickupType::Recyclables => serializer.serialize_unit_variant("", 0, "♻️ Recyclables"),
            PickupType::Greens => serializer.serialize_unit_variant("", 1, "🌳 Greens"),
            PickupType::Trash => serializer.serialize_unit_variant("", 2, "🗑️ Trash"),
        }
    }
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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Serialize)]
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
    use crate::Error;
    use lazy_static::lazy_static;
    use scraper::Selector;

    fn create_pickup_html(name: &str, date: &str) -> String {
        format!("<div><h3>{name}</h3><p></p><p></p><p>{date}</p></div>")
    }

    lazy_static! {
        static ref SELECTOR: Selector = Selector::parse("div").unwrap();
    }

    #[test]
    fn parse_single_pickup() {
        let html = create_pickup_html("Trash", "01/01/2023");
        let result = Pickup::try_from(html.as_str()).unwrap();

        assert_eq!(
            result,
            Pickup {
                name: PickupType::Trash,
                date: NaiveDate::from_ymd(2023, 01, 01),
            }
        )
    }

    #[test]
    fn parse_single_pickup_with_bad_name() {
        let html = create_pickup_html("Not a real thing", "01/01/2023");
        let result = Pickup::try_from(html.as_str());

        assert_eq!(result, Err(Error::ParseError));
    }
}
