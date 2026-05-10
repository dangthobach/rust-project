import { For, Show, JSX } from 'solid-js';
import { Spinner } from './Spinner';

export interface ColumnDef<T> {
  key: string;
  header: string | JSX.Element;
  /** Width hint, e.g. "w-40" Tailwind class */
  width?: string;
  render: (item: T, index: number) => JSX.Element;
}

interface DataTableProps<T extends { id: string }> {
  columns: ColumnDef<T>[];
  data: T[];
  loading?: boolean;
  emptyMessage?: string;

  /** Multi-select */
  selectedIds?: Set<string>;
  onSelectionChange?: (ids: Set<string>) => void;

  /** Per-row action slot (rendered in last column) */
  rowActions?: (item: T) => JSX.Element;

  /** Called when the row body (not checkbox/actions) is clicked */
  onRowClick?: (item: T) => void;
}

/**
 * Generic table with optional multi-select and row actions.
 *
 * Generic usage (SolidJS doesn't infer generics in JSX, so pass the component
 * directly or wrap it):
 *   <DataTable<RoleDto> columns={cols} data={items()} ... />
 */
export function DataTable<T extends { id: string }>(props: DataTableProps<T>) {
  const hasSelection = () => props.selectedIds !== undefined;
  const allSelected = () =>
    props.data.length > 0 &&
    props.data.every((r) => props.selectedIds?.has(r.id));
  const someSelected = () =>
    !allSelected() && props.data.some((r) => props.selectedIds?.has(r.id));

  function toggleAll() {
    if (!props.onSelectionChange) return;
    if (allSelected()) {
      props.onSelectionChange(new Set());
    } else {
      props.onSelectionChange(new Set(props.data.map((r) => r.id)));
    }
  }

  function toggleRow(id: string) {
    if (!props.onSelectionChange || !props.selectedIds) return;
    const next = new Set(props.selectedIds);
    next.has(id) ? next.delete(id) : next.add(id);
    props.onSelectionChange(next);
  }

  return (
    <div class="overflow-x-auto border-[3px] border-black bg-white shadow-brutal-sm">
      <Show when={props.loading}>
        <div class="flex justify-center p-10">
          <Spinner />
        </div>
      </Show>

      <Show when={!props.loading && props.data.length === 0}>
        <div class="p-10 text-center font-mono text-xs uppercase tracking-widest text-neutral-darkGray">
          {props.emptyMessage ?? 'Không có dữ liệu'}
        </div>
      </Show>

      <Show when={!props.loading && props.data.length > 0}>
        <table class="w-full border-collapse font-mono text-xs">
          <thead>
            <tr class="bg-neutral-lightGray">
              <Show when={hasSelection()}>
                <th class="w-10 border-b-[3px] border-black px-3 py-2">
                  <input
                    type="checkbox"
                    checked={allSelected()}
                    indeterminate={someSelected()}
                    onChange={toggleAll}
                    class="h-3.5 w-3.5 cursor-pointer accent-black"
                    aria-label="Chọn tất cả"
                  />
                </th>
              </Show>

              <For each={props.columns}>
                {(col) => (
                  <th
                    class={`border-b-[3px] border-black px-3 py-2 text-left font-heading text-[10px] font-black uppercase ${col.width ?? ''}`}
                  >
                    {col.header}
                  </th>
                )}
              </For>

              <Show when={!!props.rowActions}>
                <th class="w-24 border-b-[3px] border-black px-3 py-2 text-right font-heading text-[10px] font-black uppercase">
                  Thao tác
                </th>
              </Show>
            </tr>
          </thead>

          <tbody>
            <For each={props.data}>
              {(item, index) => {
                const isSelected = () => props.selectedIds?.has(item.id) ?? false;
                return (
                  <tr
                    class={`transition-colors ${isSelected() ? 'bg-ledger-lime/20' : 'bg-white hover:bg-neutral-lightGray/60'}`}
                  >
                    <Show when={hasSelection()}>
                      <td class="border-b-[3px] border-black px-3 py-2">
                        <input
                          type="checkbox"
                          checked={isSelected()}
                          onChange={() => toggleRow(item.id)}
                          class="h-3.5 w-3.5 cursor-pointer accent-black"
                          aria-label={`Chọn hàng ${index() + 1}`}
                        />
                      </td>
                    </Show>

                    <For each={props.columns}>
                      {(col) => (
                        <td
                          class={`border-b-[3px] border-black px-3 py-3 ${props.onRowClick ? 'cursor-pointer' : ''}`}
                          onClick={() => props.onRowClick?.(item)}
                        >
                          {col.render(item, index())}
                        </td>
                      )}
                    </For>

                    <Show when={!!props.rowActions}>
                      <td class="border-b-[3px] border-black px-3 py-2 text-right">
                        {props.rowActions!(item)}
                      </td>
                    </Show>
                  </tr>
                );
              }}
            </For>
          </tbody>
        </table>
      </Show>
    </div>
  );
}
