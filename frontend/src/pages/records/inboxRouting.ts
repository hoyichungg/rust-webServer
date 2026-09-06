export const INBOX_STATES = ["unread", "read", "snoozed", "dismissed", "all", "archived"];
export type InboxQuery = { state: string; search: string; source: string; severity: string; page: number };

export function parseInboxQuery(params: URLSearchParams): InboxQuery {
  const page = Number(params.get("page") || 1);
  const state = params.get("state") || "unread";
  const severity = params.get("severity") || "";
  return {
    state: INBOX_STATES.includes(state) ? state : "unread",
    search: (params.get("search") || "").trim().slice(0, 200),
    source: (params.get("source") || "").trim().slice(0, 64),
    severity: ["info", "warning", "critical"].includes(severity) ? severity : "",
    page: Number.isSafeInteger(page) && page >= 1 && page <= 1_000_000 ? page : 1
  };
}

export function inboxParams(query: InboxQuery): URLSearchParams {
  const params = new URLSearchParams({ state: query.state });
  for (const key of ["search", "source", "severity"] as const) {
    if (query[key]) params.set(key, query[key]);
  }
  params.set("page", String(query.page));
  return params;
}

export function inboxHash(query: InboxQuery): string {
  return `#inbox?${inboxParams(query)}`;
}

export function inboxReturnHash(params: URLSearchParams): string {
  return params.get("from") === "inbox" ? inboxHash(parseInboxQuery(params)) : "#dashboard";
}
