import { useMemo } from 'react';
import { Download, Loader2, Search } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Card } from '@/components/ui/Card';
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
import { SortHeader, providerLabel, type HeatmapRow } from './shared';
import type { HeatmapCell } from '@/types';

const GOTHER = 'gother';

/**
 * Hotel x provider grid for each hotel's most recent stay, with the
 * cheapest provider highlighted.
 *
 * Columns come from the sources actually present in the data, except Gother:
 * that one is pinned first and always rendered, even with no data. This grid
 * exists to answer "where do we sit against the market", and a Gother column
 * that vanishes when empty answers it by hiding the question — matching how
 * Market Position already treats it.
 *
 * (The earlier global-page version hardcoded ['agoda','trip','wink'], which
 * silently omitted six providers that had data and showed one that never will.)
 */
export function CompetitorHeatmapCard({
  cells,
  isLoading,
  exportName,
}: {
  cells: HeatmapCell[] | undefined;
  isLoading?: boolean;
  exportName: string;
}) {
  const heatmapSources = useMemo(() => {
    const seen = new Set<string>();
    (cells ?? []).forEach((c) => seen.add(c.source));
    seen.delete(GOTHER);
    // Gother first — the reference price you read the rest of the row against.
    return [GOTHER, ...Array.from(seen).sort()];
  }, [cells]);

  /** True while no hotel anywhere has a Gother price, which is the state today. */
  const gotherIsEmpty = useMemo(
    () => !(cells ?? []).some((c) => c.source === GOTHER && c.price_thb != null),
    [cells]
  );

  const heatmapRows: HeatmapRow[] = useMemo(() => {
    const byHotel = new Map<string, HeatmapRow>();
    (cells ?? []).forEach((cell: HeatmapCell) => {
      const row =
        byHotel.get(cell.hotel_id) ??
        ({
          hotel_id: cell.hotel_id,
          hotel_name: cell.hotel_name,
          checkin_date: cell.checkin_date,
          prices: {},
          cheapestSource: null,
        } as HeatmapRow);
      row.prices[cell.source] = cell.price_thb;
      if (cell.is_cheapest) row.cheapestSource = cell.source;
      byHotel.set(cell.hotel_id, row);
    });
    return Array.from(byHotel.values());
  }, [cells]);

  const heat = useTableControls('hm_', heatmapRows, {
    searchText: (r) => r.hotel_name,
    sortValues: {
      hotel: (r) => r.hotel_name,
      cheapest: (r) => (r.cheapestSource ? r.prices[r.cheapestSource] ?? null : null),
    },
    defaultSort: 'hotel',
    defaultDirection: 'asc',
  });

  return (
      <Card>
        <div className="px-5 py-3.5 border-b flex items-center justify-between gap-4">
          <div>
            <h2 className="text-sm font-bold">Competitor Heatmap</h2>
            <p className="text-xs text-muted-foreground mt-0.5">
              Price per provider for each hotel's most recent stay.{' '}
              <span className="text-green-700 font-medium">Green</span> marks the cheapest provider
              for that stay.
            </p>
            {gotherIsEmpty && (
              <p className="text-xs text-amber-700 mt-1">
                The Gother column is empty: Gother does not appear in Google Hotels, so SerpAPI
                cannot return it. It fills in once <code className="text-[11px]">GOTHER_API_URL</code>{' '}
                and <code className="text-[11px]">GOTHER_API_KEY</code> are set. Left blank rather
                than estimated — a guessed own-price would be compared against real competitor
                prices.
              </p>
            )}
          </div>
          <div className="flex items-center gap-2">
            <div className="relative">
              <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
              <Input
                value={heat.search}
                onChange={(e) => heat.setSearch(e.target.value)}
                placeholder="Search hotels..."
                className="h-8 pl-8 w-52 text-xs"
              />
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={heat.filtered.length === 0}
              onClick={() =>
                exportCsv(
                  `${exportName.replace(/\s+/g, '-')}-competitor-heatmap.csv`,
                  ['hotel', 'checkin_date', 'cheapest_provider', ...heatmapSources],
                  heat.filtered,
                  (r) => [
                    r.hotel_name,
                    r.checkin_date,
                    r.cheapestSource ?? '',
                    ...heatmapSources.map((s) => r.prices[s]?.toFixed(2) ?? ''),
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
        ) : heat.filtered.length > 0 ? (
          <>
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <SortHeader
                      label="Hotel"
                      sortKey="hotel"
                      active={heat.sortKey}
                      direction={heat.direction}
                      onSort={heat.toggleSort}
                    />
                    <TableHead>Stay</TableHead>
                    {heatmapSources.map((source) => (
                      <TableHead
                        key={source}
                        className={`text-right whitespace-nowrap ${
                          source === GOTHER ? 'text-brand-600 border-r' : ''
                        }`}
                      >
                        {providerLabel(source)}
                      </TableHead>
                    ))}
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {heat.pageRows.map((r) => (
                    <TableRow key={r.hotel_id}>
                      <TableCell className="font-medium whitespace-nowrap">
                        <Link to={`/hotels/${r.hotel_id}`} className="hover:text-brand-600">
                          {r.hotel_name}
                        </Link>
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        {r.checkin_date}
                      </TableCell>
                      {heatmapSources.map((source) => {
                        const price = r.prices[source];
                        const isWinner = r.cheapestSource === source && price != null;
                        return (
                          <TableCell
                            key={source}
                            className={`text-right ${
                              isWinner
                                ? 'bg-green-50 text-green-800 font-semibold'
                                : 'text-muted-foreground'
                            } ${source === GOTHER ? 'border-r' : ''}`}
                          >
                            {price != null ? formatPrice(price) : '—'}
                          </TableCell>
                        );
                      })}
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
            <div className="px-5 pb-4">
              <Pagination
                page={heat.page}
                totalPages={heat.totalPages}
                totalItems={heat.filtered.length}
                pageSize={heat.pageSize}
                onPageChange={heat.setPage}
                onPageSizeChange={heat.setPageSize}
              />
            </div>
          </>
        ) : (
          <div className="text-center py-8 text-muted-foreground text-sm">
            {heat.search ? 'No hotels match that search.' : 'No price data for this group yet.'}
          </div>
        )}
      </Card>
  );
}
