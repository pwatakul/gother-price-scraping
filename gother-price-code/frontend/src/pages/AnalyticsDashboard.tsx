import { useQuery } from '@tanstack/react-query';
import { Building2, TrendingDown, Trophy, Download, AlertTriangle } from 'lucide-react';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { MarketPositionCard } from '@/components/analytics/MarketPositionCard';
import { CompetitorHeatmapCard } from '@/components/analytics/CompetitorHeatmapCard';
import {
  getOverview,
  getMarketPosition,
  getHeatmap,
  getWinRate,
  getProviderBenchmark,
  getParityViolations,
} from '@/api/analytics';
import { downloadBlob } from '@/api/scrapeJobs';
import apiClient from '@/api/client';
import { formatPrice } from '@/utils/format';
import { Badge } from '@/components/ui/Badge';
import { PROVIDER_LABELS, type Provider } from '@/utils/providers';

export function AnalyticsDashboard() {

  const { data: overview } = useQuery({ queryKey: ['analytics', 'overview'], queryFn: () => getOverview() });
  const { data: positions, isLoading: positionsLoading } = useQuery({
    queryKey: ['analytics', 'market-position'],
    queryFn: () => getMarketPosition(),
  });
  const { data: heatmapCells, isLoading: heatmapLoading } = useQuery({
    queryKey: ['analytics', 'heatmap'],
    queryFn: () => getHeatmap(),
  });
  const { data: winRates } = useQuery({ queryKey: ['analytics', 'win-rate'], queryFn: getWinRate });
  const { data: benchmark } = useQuery({
    queryKey: ['analytics', 'provider-benchmark'],
    queryFn: () => getProviderBenchmark(),
  });
  const { data: violations } = useQuery({
    queryKey: ['analytics', 'parity-violations'],
    queryFn: () => getParityViolations(),
  });

  const handleExport = async () => {
    const response = await apiClient.get('/analytics/export', { responseType: 'blob' });
    downloadBlob(response.data, 'market-position.csv');
  };

  return (
    <div className="max-w-[1400px] mx-auto py-6 px-7">
      <div className="flex items-start justify-between mb-6">
        <div>
          <h1 className="text-xl font-bold">📈 Market Analytics</h1>
          <p className="text-sm text-muted-foreground mt-1">
            Where Gother wins, where it loses, and how the gap is trending — all from historical scrape data.
          </p>
        </div>
        <Button onClick={handleExport}>
          <Download className="h-4 w-4 mr-2" />
          Export Report
        </Button>
      </div>

      {violations && violations.length > 0 && (
        <div className="flex items-start gap-2.5 rounded-lg bg-red-50 border border-red-200 text-red-800 px-4 py-3 mb-5 text-sm">
          <AlertTriangle className="h-4 w-4 mt-0.5 shrink-0" />
          <div>
            <strong>{violations.length} rate parity violation{violations.length !== 1 ? 's' : ''}</strong> —
            Gother is more expensive than the best OTA by more than 5% on:{' '}
            {violations.slice(0, 3).map((v) => v.hotel_name).join(', ')}
            {violations.length > 3 && ` and ${violations.length - 3} more`}.
          </div>
        </div>
      )}

      {/* Overview KPIs */}
      <div className="grid grid-cols-3 gap-3 mb-6">
        <Card className="p-4">
          <div className="flex items-center gap-3">
            <div className="h-9 w-9 rounded-full bg-brand-50 flex items-center justify-center">
              <Building2 className="h-4 w-4 text-brand-600" />
            </div>
            <div>
              <div className="text-[26px] font-bold leading-none">{overview?.total_hotels ?? '—'}</div>
              <div className="text-xs text-muted-foreground mt-1">Hotels Tracked</div>
            </div>
          </div>
        </Card>
        <Card className="p-4">
          <div className="flex items-center gap-3">
            <div className="h-9 w-9 rounded-full bg-green-50 flex items-center justify-center">
              <Trophy className="h-4 w-4 text-green-600" />
            </div>
            <div>
              <div className="text-[26px] font-bold leading-none text-green-600">
                {overview ? `${overview.gother_cheapest_pct.toFixed(0)}%` : '—'}
              </div>
              <div className="text-xs text-muted-foreground mt-1">Gother Win Rate</div>
              <div className="text-[10px] text-muted-foreground/70">awaiting Gother API</div>
            </div>
          </div>
        </Card>
        <Card className="p-4">
          <div className="flex items-center gap-3">
            <div className="h-9 w-9 rounded-full bg-red-50 flex items-center justify-center">
              <TrendingDown className="h-4 w-4 text-red-600" />
            </div>
            <div>
              <div className="text-[26px] font-bold leading-none">
                {overview ? formatPrice(overview.avg_gap_thb) : '—'}
              </div>
              <div className="text-xs text-muted-foreground mt-1">Avg Price Gap</div>
              <div className="text-[10px] text-muted-foreground/70">awaiting Gother API</div>
            </div>
          </div>
        </Card>
      </div>

      {/* Provider benchmark — the one comparison that does not depend on a
          Gother price, which has no data source yet. */}
      <Card className="mb-4">
        <div className="px-5 py-3.5 border-b">
          <h2 className="text-sm font-bold">Provider Benchmark</h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Providers compared only within identical stays — same hotel, same check-in date — so
            the comparison is like for like. Stays quoted by a single provider are excluded.
            Measured from scraped prices, independent of Gother.
          </p>
        </div>
        {benchmark && benchmark.length > 0 ? (
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
                  <TableCell className="font-medium">
                    <Badge variant="info">
                      {PROVIDER_LABELS[r.source as Provider] ?? r.source}
                    </Badge>
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
            No price data yet. Run a price search to populate this.
          </div>
        )}
      </Card>

      <MarketPositionCard
        entries={positions}
        isLoading={positionsLoading}
        exportName="all-groups"
      />

      <CompetitorHeatmapCard
        cells={heatmapCells}
        isLoading={heatmapLoading}
        exportName="all-groups"
      />

      <Card className="mt-4">
        <div className="px-5 py-3.5 border-b">
          <h2 className="text-sm font-bold">Win Rate by Hotel</h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Share of days Gother was the cheapest source. Populates once the Gother API is
            connected — until then there is no Gother price to compare, so this is empty rather
            than zero-by-measurement.
          </p>
        </div>
        {winRates && winRates.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Hotel</TableHead>
                <TableHead className="text-right">Days Won</TableHead>
                <TableHead className="text-right">Win Rate</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {winRates.map((w) => {
                const name = positions?.find((p) => p.hotel_id === w.hotel_id)?.hotel_name ?? w.hotel_id;
                return (
                  <TableRow key={w.hotel_id}>
                    <TableCell className="font-medium">{name}</TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {w.days_won} / {w.days_total}
                    </TableCell>
                    <TableCell className="text-right font-semibold">{w.win_rate_pct.toFixed(0)}%</TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        ) : (
          <div className="text-center py-8 text-muted-foreground text-sm">
            No Gother prices yet — connect the Gother API to populate this.
          </div>
        )}
      </Card>
    </div>
  );
}

