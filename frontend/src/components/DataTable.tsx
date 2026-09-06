import { Button, Group, Pagination, ScrollArea, Stack, Table, Text, TextInput } from "@mantine/core";
import { useState, type ReactNode } from "react";

import { EmptyText } from "./EmptyText";
import { formatValue } from "../utils/format";

type DataRow = object;
type CellRenderer<Row extends DataRow> = (props: { value: unknown; row: Row }) => ReactNode;
export type DataColumn<Row extends DataRow> = [key: Extract<keyof Row, string> | string, label: string, renderer?: CellRenderer<Row>];

export function DataTable<Row extends DataRow>({ rows = [], columns, searchable = false, label = "records" }: {
  rows?: Row[];
  columns: DataColumn<Row>[];
  searchable?: boolean;
  label?: string;
}) {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const term = search.trim().toLocaleLowerCase();
  const matching = searchable && term ? rows.filter((row) => columns.some(([key]) => {
    const value = (row as Record<string, unknown>)[key];
    return value != null && String(value).toLocaleLowerCase().includes(term);
  })) : rows;
  const pages = Math.max(1, Math.ceil(matching.length / 10));
  const currentPage = Math.min(page, pages);
  const visible = searchable ? matching.slice((currentPage - 1) * 10, currentPage * 10) : matching;

  return (
    <Stack gap="sm">
      {searchable && <Group justify="space-between">
        <TextInput aria-label={`Search ${label}`} placeholder={`Search ${label}…`} value={search}
          onChange={(event) => { setSearch(event.currentTarget.value); setPage(1); }}
          style={{ flex: "1 1 220px", maxWidth: 360 }} />
        <Text size="xs" c="dimmed">{matching.length} of {rows.length} in this list</Text>
        {search && <Button variant="subtle" size="compact-sm" onClick={() => { setSearch(""); setPage(1); }}>Clear search</Button>}
      </Group>}
      {visible.length === 0 ? <EmptyText>{search ? "No matches. Try another search or clear the filter." : "No records"}</EmptyText> : <ScrollArea>
      <Table striped highlightOnHover verticalSpacing="sm">
        <Table.Thead>
          <Table.Tr>
            {columns.map(([, label]) => (
              <Table.Th key={label}>{label}</Table.Th>
            ))}
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {visible.map((row, index) => {
            const rowData = row as Record<string, unknown>;
            const rowKey = rowData.id || `${rowData.source || "row"}-${index}`;

            return (
              <Table.Tr key={String(rowKey)}>
                {columns.map(([key, label, Renderer]) => (
                  <Table.Td key={`${key}-${label}`}>
                    {Renderer ? (
                      <Renderer value={rowData[key]} row={row} />
                    ) : (
                      formatValue(rowData[key])
                    )}
                  </Table.Td>
                ))}
              </Table.Tr>
            );
          })}
        </Table.Tbody>
      </Table>
      </ScrollArea>}
      {searchable && pages > 1 && <Group justify="space-between">
        <Text size="xs" c="dimmed">Page {currentPage} of {pages}</Text>
        <Pagination aria-label={`${label} pages`} total={pages} value={currentPage} onChange={setPage} size="sm" />
      </Group>}
    </Stack>
  );
}
