import { describe, it, expect } from "vitest";
import fetch from "cross-fetch";
import * as dotenv from "dotenv";
dotenv.config();

const baseUrl = "https://trashcal.stabbylambda.com";
const id = process.env["TRASHCAL_ID"] ?? "a4Ot0000001E8i4EAC";

describe("post deploy", () => {
  it("works with a basic calendar", async () => {
    let response = await fetch(`${baseUrl}/${id}`);
    let body = await response.text();
    console.log(body);

    expect(response.headers.get("content-type")).toBe(
      "text/calendar;charset=UTF-8"
    );
    expect(body).toContain("BEGIN:VCALENDAR");
    expect(body).toContain("END:VCALENDAR");
  });

  it("works with a json calendar", async () => {
    let response = await fetch(`${baseUrl}/${id}`, {
      headers: {
        accept: "application/json",
      },
    });
    let body = await response.json();
    console.log(body);

    expect(response.headers.get("content-type")).toBe("application/json");
    expect(body.id).toBe(id);
    expect(body.pickups.length).toBeGreaterThan(0);
  });
});
