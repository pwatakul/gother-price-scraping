import { Link, useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeft, Loader2 } from 'lucide-react';
import { Card } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { getHotelGroup } from '@/api/hotelGroups';
import { getHeatmap, getMarketPosition, getProviderBenchmark } from '@/api/analytics';
import { providerLabel } from '@/components/analytics/shared';
import { MarketPositionCard } from '@/components/analytics/MarketPositionCard';
import { CompetitorHeatmapCard } from '@/components/analytics/CompetitorHeatmapCard';

export function GroupAnalytics() {
  const { id } = useParams<{ id: string }>();

  const { data: group } = useQuery({
    queryKey: ['hotelGroup', id],
    queryFn: () => getHotelGroup(id!),
    enabled: !!id,
  });

  const { data: benchmark, isLoading: benchmarkLoading } = useQuery({
    queryKey: ['analytics', 'provider-benchmark', id],
    queryFn: () => getProviderBenchmark(id!),
    enabled: !!id,
  });

  const { data: positions, isLoading: positionsLoading } = useQuery({
    queryKey: ['analytics', 'market-position', id],
    queryFn: () => getMarketPosition(id!),
    enabled: !!id,
  });

  const { data: heatmapCells, isLoading: heatmapLoading } = useQuery({
    queryKey: ['analytics', 'heatmap', id],
    queryFn: () => getHeatmap(id!),
    enabled: !!id,
  });

  const groupName = group?.group.name ?? 'Group';

  return (
    <div className="max-w-[1400px] mx-auto py-6 px-7">
      <Link
        to={`/groups/${id}`}
        className="inline-flex items-center gap-1.5 text-sm text-slate-500 hover:text-slate-900 mb-3.5"
      >
        <ArrowLeft className="h-3.5 w-3.5" />
        Back to {groupName}
      </Link>

      <div className="mb-6">
        <h1 className="text-xl font-bold">📈 {groupName} — Analytics</h1>
        <p className="text-sm text-muted-foreground mt-1">
          Every figure below covers only this group's hotels. Providers are compared within a
          single stay — same hotel, same check-in date — so the comparison is like for like.
        </p>
      </div>

      {/* Provider Benchmark */}
      <Card className="mb-4">
        <div className="px-5 py-3.5 border-b">
          <h2 className="text-sm font-bold">Provider Benchmark</h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            How often each provider is the cheapest quote, and its median premium over the
            cheapest. Stays quoted by a single provider are excluded.
          </p>
        </div>
        {benchmarkLoading ? (
          <div className="flex items-center justify-center py-10">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : benchmark && benchmark.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Provider</TableHead>
                <TableHead className="text-right">Quotes compared</TableHead>
                <TableHead className="text-right">Hotels covered</TableHead>
                <TableHead className="text-right">Times cheapest</TableHead>
                <TableHead className="text-right">Cheapest</TableHead>
                <TableHead className="text-right">Median premium</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {benchmark.map((r) => (
                <TableRow key={r.source}>
                  <TableCell>
                    <Badge variant="info">{providerLabel(r.source)}</Badge>
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    {r.quotes_compared}
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    {r.hotels_covered}
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    {r.times_cheapest}
                  </TableCell>
                  <TableCell className="text-right font-semibold">{r.cheapest_pct}%</TableCell>
                  <TableCell className="text-right">
                    {r.median_premium_pct === 0 ? (
                      <span className="text-green-700 font-semibold">cheapest</span>
                    ) : (
                      `+${r.median_premium_pct}%`
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <div className="text-center py-8 text-muted-foreground text-sm">
            No price data for this group yet. Run a price search to populate this.
          </div>
        )}
      </Card>

      <MarketPositionCard
        entries={positions}
        isLoading={positionsLoading}
        exportName={groupName}
      />

      <CompetitorHeatmapCard
        cells={heatmapCells}
        isLoading={heatmapLoading}
        exportName={groupName}
      />
    </div>
  );
}
