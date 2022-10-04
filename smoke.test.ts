import { describe, it, expect } from "vitest";
import fetch from "cross-fetch";

const baseUrl = "https://trashcal.stabbylambda.com";

describe("post deploy", () => {
  it("works with a basic calendar", async () => {
    let response = await fetch(`${baseUrl}/a4Ot0000001E8i4EAC`);
    let body = await response.text();
    console.log(body);

    expect(response.headers.get("content-type")).toBe(
      "text/calendar;charset=UTF-8"
    );
    expect(body).toContain("BEGIN:VCALENDAR");
    expect(body).toContain("END:VCALENDAR");
  });

  it("works with a json calendar", async () => {
    let response = await fetch(`${baseUrl}/a4Ot0000001E8i4EAC`, {
      headers: {
        accept: "application/json",
      },
    });
    let body = await response.json();
    console.log(body);

    expect(response.headers.get("content-type")).toBe("application/json");
    expect(body.id).toBe("a4Ot0000001E8i4EAC");
    expect(body.address).toBe("1234 AGATE ST, San Diego, CA 92109");
    expect(body.pickups.length).toBeGreaterThan(0);
  });
});
