import { Download, Loader2, Search } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Card } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { Pagination } from '@/components/Pagination';
import { formatPrice } from '@/utils/format';
import { exportCsv, useTableControls } from '@/hooks/useTableControls';
import { SortHeader, providerLabel } from './shared';
import type { MarketPositionEntry } from '@/types';

/**
 * One row per hotel, compared on its most recent stay, with search,
 * sortable columns, pagination and CSV export of the filtered view.
 *
 * Shared by the global and per-group analytics pages — they render the
 * same data and previously did so through two different code paths, only
 * one of which was correct. Takes rows as props and does no fetching, so
 * each page keeps control of its own scope.
 */
export function MarketPositionCard({
  entries,
  isLoading,
  exportName,
}: {
  entries: MarketPositionEntry[] | undefined;
  isLoading?: boolean;
  /** Filename stem for the CSV, e.g. the group name. */
  exportName: string;
}) {
  const position = useTableControls('mp_', entries, {
    searchText: (r) => r.hotel_name,
    sortValues: {
      hotel: (r) => r.hotel_name,
      cheapest_price: (r) => r.cheapest_price,
      cheapest_source: (r) => r.cheapest_source,
      providers: (r) => r.provider_count,
      spread: (r) => r.spread_pct,
      gother: (r) => r.gother_price,
      gap: (r) => r.gap_pct,
    },
    defaultSort: 'spread',
  });

  return (
      <Card className="mb-4">
        <div className="px-5 py-3.5 border-b flex items-center justify-between gap-4">
          <div>
            <h2 className="text-sm font-bold">Market Position</h2>
            <p className="text-xs text-muted-foreground mt-0.5">
              One row per hotel, compared on its most recent stay. Gother columns stay empty until
              its API is connected.
            </p>
          </div>
          <div className="flex items-center gap-2">
            <div className="relative">
              <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
              <Input
                value={position.search}
                onChange={(e) => position.setSearch(e.target.value)}
                placeholder="Search hotels..."
                className="h-8 pl-8 w-52 text-xs"
              />
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={position.filtered.length === 0}
              onClick={() =>
                exportCsv(
                  `${exportName.replace(/\s+/g, "-")}-market-position.csv`,
                  [
                    'hotel',
                    'checkin_date',
                    'cheapest_source',
                    'cheapest_price_thb',
                    'providers_quoting',
                    'spread_pct',
                    'gother_price_thb',
                    'best_price_thb',
                    'best_source',
                    'gap_thb',
                    'gap_pct',
                  ],
                  position.filtered,
                  (r) => [
                    r.hotel_name,
                    r.checkin_date,
                    r.cheapest_source,
                    r.cheapest_price?.toFixed(2),
                    r.provider_count,
                    r.spread_pct?.toFixed(1),
                    r.gother_price?.toFixed(2),
                    r.best_price?.toFixed(2),
                    r.best_source,
                    r.gap_thb?.toFixed(2),
                    r.gap_pct?.toFixed(1),
                  ]
                )
              }
            >
              <Download className="h-3.5 w-3.5 mr-1.5" />
              Export
            </Button>
          </div>
        </div>
        {isLoading ? (
          <div className="flex items-center justify-center py-10">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : position.filtered.length > 0 ? (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <SortHeader
                    label="Hotel"
                    sortKey="hotel"
                    active={position.sortKey}
                    direction={position.direction}
                    onSort={position.toggleSort}
                  />
                  <TableHead>Stay</TableHead>
                  <SortHeader
                    label="Cheapest"
                    sortKey="cheapest_source"
                    active={position.sortKey}
                    direction={position.direction}
                    onSort={position.toggleSort}
                  />
                  <SortHeader
                    label="Price"
                    sortKey="cheapest_price"
                    active={position.sortKey}
                    direction={position.direction}
                    onSort={position.toggleSort}
                    align="right"
                  />
                  <SortHeader
                    label="Providers"
                    sortKey="providers"
                    active={position.sortKey}
                    direction={position.direction}
                    onSort={position.toggleSort}
                    align="right"
                  />
                  <SortHeader
                    label="Spread"
                    sortKey="spread"
                    active={position.sortKey}
                    direction={position.direction}
                    onSort={position.toggleSort}
                    align="right"
                  />
                  <SortHeader
                    label="Gother"
                    sortKey="gother"
                    active={position.sortKey}
                    direction={position.direction}
                    onSort={position.toggleSort}
                    align="right"
                  />
                  <SortHeader
                    label="Gap"
                    sortKey="gap"
                    active={position.sortKey}
                    direction={position.direction}
                    onSort={position.toggleSort}
                    align="right"
                  />
                </TableRow>
              </TableHeader>
              <TableBody>
                {position.pageRows.map((r) => (
                  <TableRow key={r.hotel_id}>
                    <TableCell className="font-medium">
                      <Link to={`/hotels/${r.hotel_id}`} className="hover:text-brand-600">
                        {r.hotel_name}
                      </Link>
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {r.checkin_date}
                    </TableCell>
                    <TableCell>
                      {r.cheapest_source ? (
                        <Badge variant="success">{providerLabel(r.cheapest_source)}</Badge>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </TableCell>
                    <TableCell className="text-right font-semibold">
                      {r.cheapest_price != null ? formatPrice(r.cheapest_price) : '—'}
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {r.provider_count}
                    </TableCell>
                    <TableCell className="text-right">
                      {r.spread_pct != null ? `${r.spread_pct.toFixed(1)}%` : '—'}
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {r.gother_price != null ? formatPrice(r.gother_price) : '—'}
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {r.gap_pct != null ? `${r.gap_pct.toFixed(1)}%` : '—'}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
            <div className="px-5 pb-4">
              <Pagination
                page={position.page}
                totalPages={position.totalPages}
                totalItems={position.filtered.length}
                pageSize={position.pageSize}
                onPageChange={position.setPage}
                onPageSizeChange={position.setPageSize}
              />
            </div>
          </>
        ) : (
          <div className="text-center py-8 text-muted-foreground text-sm">
            {position.search ? 'No hotels match that search.' : 'No price data for this group yet.'}
          </div>
        )}
      </Card>
  );
}
