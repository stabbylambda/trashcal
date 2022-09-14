# 🗑️📆 Trashcal

Trashcal is a small AWS Lambda function that parses the City of San Diego's trash collection
pages and produces an iCal file with events for trash, recycling, and greens days.

The [actual lambda](./lambda/README.md) is written in Rust and you'll need [cargo lambda](https://www.cargo-lambda.info) to build it. 

## Usage

This is a CDK app, so set up all your AWS stuff, set `DOMAIN_NAME` to the domain you want, and then run `cdk deploy`.

## How it works

San Diego has all of the waste pickup info in [a Salesforce app](https://getitdone.force.com/apex/CollectionMapLookup) where you can
get either your PDF calendar with alternate Recycling and Greens weeks or get an HTML page with [your next pickup dates](https://getitdone.force.com/CollectionDetail?id=a4Ot0000001E8i4EAC). (For the record, I don't know who lives at 1234 Agate St. It was just the first address that popped up
when I searched `1234 A`). This lambda just loads that page, parses the HTML, and generates the `ICalCalendar` for you.

There is a JSON representation of this information, but that would require parsing HTML to get the CSRF token anyway and I
worry that the Salesforce API would change. The city calls this a "bookmarkable page" so I believe this will be durable.
