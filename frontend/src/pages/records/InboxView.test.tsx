import { fireEvent, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { renderWithProviders } from "../../test/render";
import { createMockApiClient } from "../../test/mockApiClient";
import { InboxView } from "./InboxView";
import { inboxReturnHash, parseInboxQuery } from "./inboxRouting";

const item = { id: 42, source: "erp", title: "Approve access", body: "Request details", severity: "warning", is_read: false, source_is_read: false, updated_at: "2026-09-06T10:00:00Z", archived_at: null, dismissed_at: "2026-09-06T10:00:00Z", snoozed_until: null };
const path = "/me/notifications?state=dismissed&page=2&page_size=25";

describe("Inbox", () => {
  it("loads URL state and preserves it through detail navigation", async () => {
    const { client } = createMockApiClient({ [path]: { items: [item], total: 26, page: 2, page_size: 25 } });
    const open = vi.fn();
    const navigate = vi.fn();
    renderWithProviders(<InboxView client={client} searchParams={new URLSearchParams("state=dismissed&page=2")} onNavigate={navigate} onOpenNotification={open} />);
    expect(await screen.findByText("Approve access")).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: "Details" }));
    expect(open).toHaveBeenCalledWith(42, "#notifications/42?from=inbox&state=dismissed&page=2");
    fireEvent.change(screen.getByLabelText("Search title or message"), { target: { value: "access" } });
    await userEvent.click(screen.getByRole("button", { name: "Search" }));
    expect(navigate).toHaveBeenCalledWith("#inbox?state=dismissed&search=access&page=1");
  });

  it("restores a dismissed message and reloads the list", async () => {
    let restored = false;
    const { client, calls } = createMockApiClient({
      [path]: () => ({ items: restored ? [] : [item], total: restored ? 0 : 26, page: 2, page_size: 25 }),
      "POST /notifications/42/restore": () => { restored = true; return { ...item, dismissed_at: null }; }
    });
    const navigate = vi.fn();
    renderWithProviders(<InboxView client={client} searchParams={new URLSearchParams("state=dismissed&page=2")} onNavigate={navigate} onOpenNotification={vi.fn()} />);
    await userEvent.click(await screen.findByRole("button", { name: "Restore" }));
    await waitFor(() => expect(calls.filter((call) => call.method === "GET")).toHaveLength(2));
    expect(await screen.findByText(/Restored\. Read status/)).toBeVisible();
    expect(navigate).toHaveBeenCalledWith("#inbox?state=dismissed&page=1");
  });

  it("keeps the record available when an action fails", async () => {
    const { client } = createMockApiClient({ [path]: { items: [item], total: 26 }, "POST /notifications/42/restore": () => { throw new Error("Try again later"); } });
    renderWithProviders(<InboxView client={client} searchParams={new URLSearchParams("state=dismissed&page=2")} onNavigate={vi.fn()} onOpenNotification={vi.fn()} />);
    await userEvent.click(await screen.findByRole("button", { name: "Restore" }));
    expect(await screen.findByText("Try again later")).toBeVisible();
    expect(screen.getByText("Approve access")).toBeVisible();
    expect(screen.getByRole("button", { name: "Restore" })).toBeEnabled();
  });

  it("does not offer source read overrides or source archive restoration", async () => {
    const { client } = createMockApiClient({ [path]: { items: [{ ...item, archived_at: item.updated_at, source_is_read: true, is_read: true }], total: 26 } });
    renderWithProviders(<InboxView client={client} searchParams={new URLSearchParams("state=dismissed&page=2")} onNavigate={vi.fn()} onOpenNotification={vi.fn()} />);
    expect(await screen.findByText("Read in source system")).toBeVisible();
    expect(screen.queryByRole("button", { name: "Restore" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Mark unread" })).not.toBeInTheDocument();
  });

  it("normalizes invalid URLs and only permits internal return routes", () => {
    expect(parseInboxQuery(new URLSearchParams("state=invalid&page=-9&severity=bogus"))).toEqual({ state: "unread", page: 1, search: "", source: "", severity: "" });
    expect(inboxReturnHash(new URLSearchParams("from=https://evil.test"))).toBe("#dashboard");
    expect(inboxReturnHash(new URLSearchParams("from=inbox&state=snoozed&page=2&token=secret"))).toBe("#inbox?state=snoozed&page=2");
  });
});
