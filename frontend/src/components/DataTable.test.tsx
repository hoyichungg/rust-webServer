import { fireEvent, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { renderWithProviders } from "../test/render";
import { DataTable } from "./DataTable";

describe("Searchable lists", () => {
  it("finds a record outside the first page and lets the user clear the filter", async () => {
    const rows = Array.from({ length: 25 }, (_, index) => ({ id: index, name: `Service ${index}` }));
    renderWithProviders(<DataTable rows={rows} columns={[["name", "Service"]]} searchable label="services" />);
    expect(screen.queryByText("Service 24")).not.toBeInTheDocument();
    fireEvent.change(screen.getByRole("textbox", { name: "Search services" }), { target: { value: "SERVICE 24" } });
    expect(screen.getByText("Service 24")).toBeVisible();
    expect(screen.getByText("1 of 25 in this list")).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: "Clear search" }));
    expect(screen.getByText("Service 0")).toBeVisible();
    expect(screen.getByText("Page 1 of 3")).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: "3" }));
    expect(screen.getByText("Service 24")).toBeVisible();
  });

  it("keeps the search available when nothing matches", () => {
    renderWithProviders(<DataTable rows={[{ id: 1, name: "API" }]} columns={[["name", "Service"]]} searchable label="services" />);
    fireEvent.change(screen.getByRole("textbox", { name: "Search services" }), { target: { value: "missing" } });
    expect(screen.getByText(/No matches/)).toBeVisible();
    expect(screen.getByRole("button", { name: "Clear search" })).toBeEnabled();
  });
});
