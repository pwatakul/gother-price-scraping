import { useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';

export type SortDirection = 'asc' | 'desc';

/**
 * Search + sort + pagination for a client-side table, with all state in
 * URL params so a filtered view survives a reload and can be shared.
 *
 * `prefix` namespaces the params, so two tables on one page (Market
 * Position and the Heatmap) don't fight over `?q=` and `?page=`.
 */
export function useTableControls<T>(
  prefix: string,
  rows: T[] | undefined,
  options: {
    /** Fields matched against the search box. */
    searchText: (row: T) => string;
    /** Sortable columns, by key. Return null for "no value" — those sort last. */
    sortValues: Record<string, (row: T) => string | number | null>;
    defaultSort: string;
    defaultDirection?: SortDirection;
    defaultPageSize?: number;
  }
) {
  const [searchParams, setSearchParams] = useSearchParams();

  const key = (name: string) => `${prefix}${name}`;
  const search = searchParams.get(key('q')) ?? '';
  const sortKey = searchParams.get(key('sort')) ?? options.defaultSort;
  const direction = (searchParams.get(key('dir')) as SortDirection) ?? options.defaultDirection ?? 'desc';
  const page = Number(searchParams.get(key('page')) ?? '0');
  const pageSize = Number(searchParams.get(key('size')) ?? String(options.defaultPageSize ?? 25));

  const update = (updates: Record<string, string | number | undefined>, resetPage = true) => {
    const next = new URLSearchParams(searchParams);
    for (const [name, value] of Object.entries(updates)) {
      if (value === undefined || value === '') next.delete(key(name));
      else next.set(key(name), String(value));
    }
    if (resetPage) next.delete(key('page'));
    setSearchParams(next);
  };

  /** Clicking the active column flips direction; a new column starts descending. */
  const toggleSort = (nextKey: string) => {
    if (nextKey === sortKey) {
      update({ dir: direction === 'asc' ? 'desc' : 'asc' }, false);
    } else {
      update({ sort: nextKey, dir: 'desc' }, false);
    }
  };

  const filtered = useMemo(() => {
    const all = rows ?? [];
    const q = search.trim().toLowerCase();
    const matched = q ? all.filter((r) => options.searchText(r).toLowerCase().includes(q)) : all;

    const valueOf = options.sortValues[sortKey];
    if (!valueOf) return matched;

    return [...matched].sort((a, b) => {
      const av = valueOf(a);
      const bv = valueOf(b);
      // Missing values sort last in both directions — most Gother columns
      // are null, and burying them under a descending sort is not useful.
      if (av === null && bv === null) return 0;
      if (av === null) return 1;
      if (bv === null) return -1;
      const cmp = typeof av === 'number' && typeof bv === 'number'
        ? av - bv
        : String(av).localeCompare(String(bv));
      return direction === 'asc' ? cmp : -cmp;
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rows, search, sortKey, direction]);

  const totalPages = Math.ceil(filtered.length / pageSize);
  // Clamp: narrowing the search can leave the URL pointing past the end.
  const safePage = Math.min(page, Math.max(totalPages - 1, 0));
  const pageRows = filtered.slice(safePage * pageSize, (safePage + 1) * pageSize);

  return {
    search,
    setSearch: (value: string) => update({ q: value }),
    sortKey,
    direction,
    toggleSort,
    page: safePage,
    pageSize,
    totalPages,
    setPage: (p: number) => update({ page: p }, false),
    setPageSize: (size: number) => update({ size }),
    /** Everything matching the search, in sort order — what export writes. */
    filtered,
    /** Just the current page — what the table renders. */
    pageRows,
  };
}

/** Quote a CSV field only when it needs it. */
function csvCell(value: unknown): string {
  if (value === null || value === undefined) return '';
  const s = String(value);
  return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
}

/**
 * Build and download a CSV from rows already filtered and sorted, so the
 * file matches exactly what is on screen.
 */
export function exportCsv<T>(
  filename: string,
  headers: string[],
  rows: T[],
  toCells: (row: T) => unknown[]
) {
  const lines = [headers.join(','), ...rows.map((r) => toCells(r).map(csvCell).join(','))];
  const blob = new Blob([lines.join('\n')], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}
