import { Alert, Badge, Button, Group, Pagination, Paper, Select, Stack, Text, TextInput } from "@mantine/core";
import { useEffect, useMemo, useRef, useState } from "react";
import type { ApiClient } from "../../api/client";
import { ViewFrame } from "../../components/ViewFrame";
import { StatusBadge } from "../../components/tableCells";
import { useAsyncData } from "../../hooks/useAsyncData";
import { useRefresh } from "../../hooks/useRefresh";
import type { ApiId, Notification } from "../../types/api";
import { inboxHash, inboxParams, parseInboxQuery, type InboxQuery } from "./inboxRouting";
import { performNotificationAction, snoozeUntilForPreset, type NotificationAction } from "./notificationActions";

type InboxResponse = { items: Notification[]; total: number; page: number; page_size: number };

export function InboxView({ client, searchParams, onNavigate, onOpenNotification }: {
  client: ApiClient;
  searchParams: URLSearchParams;
  onNavigate: (hash: string) => void;
  onOpenNotification: (id: ApiId, hash: string) => void;
}) {
  const search = searchParams.toString();
  const query = useMemo(() => parseInboxQuery(new URLSearchParams(search)), [search]);
  const path = `/me/notifications?${inboxParams(query)}&page_size=25`;
  const [data, actions] = useAsyncData(async () => ({
    path, response: await client.get<InboxResponse>(path)
  }), [client, path]);
  const response = data.value?.path === path ? data.value.response : null;
  const [searchText, setSearchText] = useState(query.search);
  const [sourceText, setSourceText] = useState(query.source);
  const [pending, setPending] = useState<ApiId | null>(null);
  const pendingRef = useRef(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  useRefresh(actions.reload);
  useEffect(() => { setSearchText(query.search); setSourceText(query.source); }, [query.search, query.source]);
  const pages = Math.max(1, Math.ceil((response?.total || 0) / 25));
  useEffect(() => {
    if (response && !data.loading && query.page > pages) onNavigate(inboxHash({ ...query, page: pages }));
  }, [response, data.loading, query, pages, onNavigate]);

  function update(patch: Partial<InboxQuery>) {
    onNavigate(inboxHash({ ...query, page: 1, ...patch }));
  }

  async function run(item: Notification, action: NotificationAction) {
    if (pendingRef.current) return;
    pendingRef.current = true;
    setPending(item.id); setActionError(null); setNotice(null);
    try {
      await performNotificationAction(client, item.id, action, {
        snoozedUntil: action === "snooze" ? snoozeUntilForPreset("one-hour") : undefined
      });
      setNotice(action === "restore" ? "Restored. Read status is preserved; check Unread or Read." : "Notification updated for your account.");
      await actions.reload();
    } catch (error) {
      setActionError(error instanceof Error ? error.message : String(error));
    } finally {
      pendingRef.current = false; setPending(null);
    }
  }

  return <ViewFrame eyebrow="Personal messages" title="Inbox" error={data.error}
    loading={data.loading && !response}
    actions={<Button variant="default" loading={data.loading} onClick={() => void actions.reload()}>Refresh</Button>}>
    <Stack>
      <Text c="dimmed" size="sm">Your Outlook, ERP and system messages. Actions apply to your portal account; they do not change the source system.</Text>
      <Paper withBorder p="md">
        <Stack>
          <Group grow align="flex-end">
            <Select label="Status" allowDeselect={false} value={query.state} onChange={(state) => update({ state: state || "unread" })}
              data={[{ value: "unread", label: "Unread" }, { value: "read", label: "Read" }, { value: "snoozed", label: "Snoozed" }, { value: "dismissed", label: "Dismissed" }, { value: "all", label: "All current messages" }, { value: "archived", label: "Archived by source" }]} />
            <Select label="Severity" clearable value={query.severity || null} onChange={(severity) => update({ severity: severity || "" })} data={["info", "warning", "critical"]} />
          </Group>
          <form onSubmit={(event) => { event.preventDefault(); update({ search: searchText.trim(), source: sourceText.trim() }); }}>
            <Group align="flex-end">
              <TextInput style={{ flex: 2 }} label="Search title or message" maxLength={200} value={searchText} onChange={(event) => setSearchText(event.currentTarget.value)} />
              <TextInput style={{ flex: 1 }} label="Source (exact name)" maxLength={64} value={sourceText} onChange={(event) => setSourceText(event.currentTarget.value)} />
              <Button type="submit">Search</Button>
              <Button variant="subtle" onClick={() => { setSearchText(""); setSourceText(""); update({ search: "", source: "", severity: "", state: "unread" }); }}>Reset</Button>
            </Group>
          </form>
        </Stack>
      </Paper>
      {actionError && <Alert color="red" title="Notification action failed">{actionError}</Alert>}
      {notice && <Alert color="teal">{notice}</Alert>}
      {response && <>
        <Text size="sm" c="dimmed">{response.total} messages · Newest updated first</Text>
        {response.items.length === 0 && <Text c="dimmed">No messages match these filters.</Text>}
        {response.items.map((item) => <Paper key={item.id} withBorder p="md">
          <Stack gap="xs">
            <Group><StatusBadge value={item.severity} /><Badge variant="light">{item.is_read ? "Read" : "Unread"}</Badge><Text size="sm" c="dimmed">{item.source}</Text>
              {item.dismissed_at && <Badge color="gray">Dismissed</Badge>}
              {item.archived_at && <Badge color="gray">Archived by source</Badge>}
            </Group>
            <Text fw={700}>{item.title}</Text>
            {item.body && <Text size="sm" lineClamp={3} style={{ whiteSpace: "pre-wrap" }}>{item.body}</Text>}
            <Text size="xs" c="dimmed">Updated {new Date(item.updated_at).toLocaleString()}</Text>
            {item.snoozed_until && new Date(item.snoozed_until) > new Date() && <Text size="sm">Snoozed until {new Date(item.snoozed_until).toLocaleString()}</Text>}
            {item.source_is_read && <Text size="xs" c="dimmed">Read in source system</Text>}
            <Group gap="xs">
              <Button size="compact-sm" variant="light" onClick={() => onOpenNotification(item.id, `#notifications/${item.id}?from=inbox&${inboxParams(query)}`)}>Details</Button>
              {!item.archived_at && <>
                {!item.source_is_read && <Button size="compact-sm" variant="default" disabled={pending !== null} onClick={() => void run(item, item.is_read ? "unread" : "read")}>{item.is_read ? "Mark unread" : "Mark read"}</Button>}
                {(item.dismissed_at || (item.snoozed_until && new Date(item.snoozed_until) > new Date())) ?
                  <Button size="compact-sm" disabled={pending !== null} loading={pending === item.id} onClick={() => void run(item, "restore")}>Restore</Button> : <>
                    <Button size="compact-sm" variant="default" disabled={pending !== null} onClick={() => void run(item, "snooze")}>Snooze 1 hour</Button>
                    <Button size="compact-sm" variant="subtle" disabled={pending !== null} onClick={() => void run(item, "dismiss")}>Dismiss</Button>
                  </>}
              </>}
            </Group>
          </Stack>
        </Paper>)}
        <Pagination total={pages} value={query.page} onChange={(page) => update({ page })} />
      </>}
    </Stack>
  </ViewFrame>;
}
