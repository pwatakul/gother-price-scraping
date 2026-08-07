import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
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
import { GapPill } from '@/components/GapPill';
import {
  getOverview,
  getMarketPosition,
  getHeatmap,
  getWinRate,
  getParityViolations,
  getBookingWindow,
  getHotelTrend,
} from '@/api/analytics';
import { downloadBlob } from '@/api/scrapeJobs';
import apiClient from '@/api/client';
import { formatPrice } from '@/utils/format';
import { PROVIDER_LABELS, type Provider } from '@/utils/providers';
import type { HeatmapCell } from '@/types';

const SOURCE_COLORS: Record<string, string> = {
  gother: '#0ea5e9',
  agoda: '#f43f5e',
  trip: '#f59e0b',
  wink: '#8b5cf6',
};

export function AnalyticsDashboard() {
  const [selectedHotelId, setSelectedHotelId] = useState<string>('');

  const { data: overview } = useQuery({ queryKey: ['analytics', 'overview'], queryFn: () => getOverview() });
  const { data: positions } = useQuery({
    queryKey: ['analytics', 'market-position'],
    queryFn: () => getMarketPosition(),
  });
  const { data: heatmapCells } = useQuery({ queryKey: ['analytics', 'heatmap'], queryFn: () => getHeatmap() });
  const { data: winRates } = useQuery({ queryKey: ['analytics', 'win-rate'], queryFn: getWinRate });
  const { data: violations } = useQuery({
    queryKey: ['analytics', 'parity-violations'],
    queryFn: () => getParityViolations(),
  });

  const hotels = useMemo(() => {
    const map = new Map<string, string>();
    positions?.forEach((p) => map.set(p.hotel_id, p.hotel_name));
    return Array.from(map.entries());
  }, [positions]);

  const effectiveHotelId = selectedHotelId || hotels[0]?.[0] || '';

  const { data: trend } = useQuery({
    queryKey: ['analytics', 'trend', effectiveHotelId],
    queryFn: () => getHotelTrend(effectiveHotelId),
    enabled: !!effectiveHotelId,
  });
  const { data: bookingWindow } = useQuery({
    queryKey: ['analytics', 'booking-window', effectiveHotelId],
    queryFn: () => getBookingWindow(effectiveHotelId),
    enabled: !!effectiveHotelId,
  });

  const trendChartData = useMemo(() => {
    if (!trend) return [];
    const byDay = new Map<string, Record<string, number | string>>();
    trend.forEach((point) => {
      const day = point.day.slice(0, 10);
      const row = byDay.get(day) ?? { day };
      row[point.source] = point.avg_price_thb;
      byDay.set(day, row);
    });
    return Array.from(byDay.values()).sort((a, b) => String(a.day).localeCompare(String(b.day)));
  }, [trend]);

  const bookingWindowChartData = useMemo(() => {
    if (!bookingWindow) return [];
    const byDays = new Map<number, Record<string, number | string>>();
    bookingWindow.forEach((point) => {
      const row = byDays.get(point.days_in_advance) ?? { days_in_advance: point.days_in_advance };
      row[point.source] = point.avg_price_thb;
      byDays.set(point.days_in_advance, row);
    });
    return Array.from(byDays.values()).sort(
      (a, b) => Number(b.days_in_advance) - Number(a.days_in_advance)
    );
  }, [bookingWindow]);

  const heatmapByHotel = useMemo(() => {
    const map = new Map<string, { hotel_name: string; cells: Map<string, HeatmapCell> }>();
    heatmapCells?.forEach((c) => {
      const entry = map.get(c.hotel_id) ?? { hotel_name: c.hotel_name, cells: new Map() };
      entry.cells.set(c.source, c);
      map.set(c.hotel_id, entry);
    });
    return Array.from(map.entries());
  }, [heatmapCells]);

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
            <div className="h-9 w-9 rounded-full bg-sky-50 flex items-center justify-center">
              <Building2 className="h-4 w-4 text-sky-600" />
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
            </div>
          </div>
        </Card>
      </div>

      {/* Trend + Booking Window charts */}
      <div className="grid grid-cols-2 gap-4 mb-6">
        <Card className="p-5">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-bold">Price Trend</h2>
            <select
              value={effectiveHotelId}
              onChange={(e) => setSelectedHotelId(e.target.value)}
              className="h-8 rounded-[7px] border border-input bg-background px-2 text-xs"
            >
              {hotels.map(([id, name]) => (
                <option key={id} value={id}>
                  {name}
                </option>
              ))}
            </select>
          </div>
          <ResponsiveContainer width="100%" height={220}>
            <LineChart data={trendChartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
              <XAxis dataKey="day" tick={{ fontSize: 11 }} />
              <YAxis tick={{ fontSize: 11 }} />
              <Tooltip />
              <Legend wrapperStyle={{ fontSize: 11 }} />
              {Object.keys(SOURCE_COLORS).map((source) => (
                <Line
                  key={source}
                  type="monotone"
                  dataKey={source}
                  name={PROVIDER_LABELS[source as Provider] ?? source}
                  stroke={SOURCE_COLORS[source]}
                  connectNulls
                  dot={false}
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        </Card>

        <Card className="p-5">
          <h2 className="text-sm font-bold mb-3">Booking Window</h2>
          <ResponsiveContainer width="100%" height={220}>
            <LineChart data={bookingWindowChartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
              <XAxis
                dataKey="days_in_advance"
                tick={{ fontSize: 11 }}
                label={{ value: 'Days before check-in', position: 'insideBottom', offset: -5, fontSize: 11 }}
              />
              <YAxis tick={{ fontSize: 11 }} />
              <Tooltip />
              <Legend wrapperStyle={{ fontSize: 11 }} />
              {Object.keys(SOURCE_COLORS).map((source) => (
                <Line
                  key={source}
                  type="monotone"
                  dataKey={source}
                  name={PROVIDER_LABELS[source as Provider] ?? source}
                  stroke={SOURCE_COLORS[source]}
                  connectNulls
                  dot={false}
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        </Card>
      </div>

      {/* Market Position + Heatmap */}
      <div className="grid grid-cols-2 gap-4">
        <Card>
          <div className="px-5 py-3.5 border-b">
            <h2 className="text-sm font-bold">Market Position</h2>
          </div>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Hotel</TableHead>
                <TableHead className="text-right">Gother</TableHead>
                <TableHead className="text-right">Best OTA</TableHead>
                <TableHead className="text-right">Gap</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {positions?.slice(0, 10).map((p) => (
                <TableRow key={p.hotel_id}>
                  <TableCell className="font-medium">{p.hotel_name}</TableCell>
                  <TableCell className="text-right">
                    {p.gother_price ? formatPrice(p.gother_price) : <span className="text-muted-foreground">—</span>}
                  </TableCell>
                  <TableCell className="text-right">
                    {p.best_price ? formatPrice(p.best_price) : <span className="text-muted-foreground">—</span>}
                  </TableCell>
                  <TableCell className="text-right">
                    <GapPill isCheapest={p.is_winning} priceDifference={p.gap_thb} priceDiffPercent={p.gap_pct} />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>

        <Card>
          <div className="px-5 py-3.5 border-b">
            <h2 className="text-sm font-bold">Competitor Heatmap</h2>
          </div>
          <div className="p-4 overflow-x-auto">
            <table className="text-xs" style={{ borderSpacing: '4px', borderCollapse: 'separate' }}>
              <thead>
                <tr>
                  <th className="text-left pr-3 pb-2 text-slate-500 font-semibold uppercase text-[10px]">Hotel</th>
                  {['agoda', 'trip', 'wink'].map((s) => (
                    <th key={s} className="pb-2 text-slate-500 font-semibold uppercase text-[10px]">
                      {PROVIDER_LABELS[s as Provider]}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {heatmapByHotel.slice(0, 10).map(([hotelId, { hotel_name, cells }]) => (
                  <tr key={hotelId}>
                    <td className="pr-3 py-1 font-medium whitespace-nowrap">{hotel_name}</td>
                    {['agoda', 'trip', 'wink'].map((source) => {
                      const cell = cells.get(source);
                      const gap = cell?.gap_pct;
                      const bg =
                        gap == null ? 'bg-slate-100 text-slate-400' : gap > 0 ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700';
                      return (
                        <td key={source} className="py-1">
                          <div className={`h-9 w-20 rounded-md flex items-center justify-center ${bg}`}>
                            {cell?.price_thb ? formatPrice(cell.price_thb) : '—'}
                          </div>
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      </div>

      {/* Win rate */}
      {winRates && winRates.length > 0 && (
        <Card className="mt-4">
          <div className="px-5 py-3.5 border-b">
            <h2 className="text-sm font-bold">Win Rate by Hotel</h2>
          </div>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Hotel</TableHead>
                <TableHead className="text-right">Days Won</TableHead>
                <TableHead className="text-right">Win Rate</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {winRates.slice(0, 10).map((w) => {
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
        </Card>
      )}
    </div>
  );
}

