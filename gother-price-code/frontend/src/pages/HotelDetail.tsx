import { useMemo } from 'react';
import { useParams, Link, useSearchParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeft, Download, ExternalLink, Loader2 } from 'lucide-react';
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
import { Card } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { Pagination } from '@/components/Pagination';
import { getHotelDetail } from '@/api/hotelDirectory';
import { listPriceHistory } from '@/api/analytics';
import apiClient from '@/api/client';
import { downloadBlob } from '@/api/scrapeJobs';
import { formatPrice } from '@/utils/format';
import { KNOWN_PROVIDERS, PROVIDER_LABELS, type Provider } from '@/utils/providers';

const HISTORY_PAGE_SIZE = 20;

const SOURCE_COLORS: Record<string, string> = {
  gother: '#0ea5e9',
  agoda: '#f43f5e',
  trip: '#f59e0b',
  wink: '#8b5cf6',
};

export function HotelDetail() {
  const { id } = useParams<{ id: string }>();
  const [searchParams, setSearchParams] = useSearchParams();
  const historySource = searchParams.get('source') ?? '';
  const historyPage = Number(searchParams.get('historyPage') ?? '0');

  const { data, isLoading } = useQuery({
    queryKey: ['hotels', 'detail', id],
    queryFn: () => getHotelDetail(id!),
    enabled: !!id,
  });

  const { data: history, isLoading: historyLoading } = useQuery({
    queryKey: ['price-history', id, historySource, historyPage],
    queryFn: () =>
      listPriceHistory({
        hotelId: id!,
        source: historySource || undefined,
        limit: HISTORY_PAGE_SIZE,
        offset: historyPage * HISTORY_PAGE_SIZE,
      }),
    enabled: !!id,
  });

  const updateHistoryParams = (updates: Record<string, string | number | undefined>, resetPage = true) => {
    const next = new URLSearchParams(searchParams);
    for (const [key, value] of Object.entries(updates)) {
      if (value === undefined || value === '') {
        next.delete(key);
      } else {
        next.set(key, String(value));
      }
    }
    if (resetPage) next.delete('historyPage');
    setSearchParams(next);
  };

  const historyTotalPages = history ? Math.ceil(history.total / HISTORY_PAGE_SIZE) : 0;

  const trendChartData = useMemo(() => {
    if (!data) return [];
    const byDay = new Map<string, Record<string, number | string>>();
    data.trend.forEach((point) => {
      const day = point.day.slice(0, 10);
      const row = byDay.get(day) ?? { day };
      row[point.source] = point.avg_price_thb;
      byDay.set(day, row);
    });
    return Array.from(byDay.values()).sort((a, b) => String(a.day).localeCompare(String(b.day)));
  }, [data]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!data) {
    return (
      <div className="max-w-[1400px] mx-auto py-6 px-7">
        <div className="text-center py-12">
          <h2 className="text-xl font-semibold mb-2">Hotel not found</h2>
        </div>
      </div>
    );
  }

  const { hotel, group_names, trend } = data;

  return (
    <div className="max-w-[1400px] mx-auto py-6 px-7">
      <Link
        to="/hotels"
        className="inline-flex items-center gap-1.5 text-sm text-slate-500 hover:text-slate-900 mb-3.5"
      >
        <ArrowLeft className="h-3.5 w-3.5" />
        Back to All Hotels
      </Link>

      <div className="flex items-start justify-between mb-6">
        <div>
          <h1 className="text-xl font-bold">🏨 {hotel.name}</h1>
          <p className="text-sm text-muted-foreground mt-1">
            {hotel.city ? `${hotel.city}, ` : ''}
            {hotel.country}
            {hotel.hid && ` · HID ${hotel.hid}`}
          </p>
          <div className="flex flex-wrap gap-1.5 mt-2">
            {group_names.map((g) => (
              <Badge key={g} variant="info">
                {g}
              </Badge>
            ))}
          </div>
        </div>
        <Button
          variant="outline"
          onClick={async () => {
            const response = await apiClient.get('/export/price-history', {
              params: { hotel_id: hotel.id, format: 'csv' },
              responseType: 'blob',
            });
            downloadBlob(response.data, `${hotel.name.replace(/\s+/g, '-')}-price-history.csv`);
          }}
        >
          <Download className="h-4 w-4 mr-2" />
          Export Price History
        </Button>
      </div>

      <div className="grid grid-cols-3 gap-4 mb-6">
        <Card className="p-4">
          <div className="text-[11px] font-semibold uppercase tracking-wide text-slate-500">Slug</div>
          <div className="text-sm mt-1">{hotel.slug ?? '—'}</div>
        </Card>
        <Card className="p-4">
          <div className="text-[11px] font-semibold uppercase tracking-wide text-slate-500">
            Supplier Type
          </div>
          <div className="text-sm mt-1">{hotel.supplier_type ?? '—'}</div>
        </Card>
        <Card className="p-4">
          <div className="text-[11px] font-semibold uppercase tracking-wide text-slate-500">
            Normalized Name
          </div>
          <div className="text-sm mt-1">{hotel.normalized_name}</div>
        </Card>
      </div>

      <Card className="p-5">
        <h2 className="text-sm font-bold mb-3">Price Trend (last 90 days)</h2>
        {trend.length === 0 ? (
          <p className="text-sm text-muted-foreground py-8 text-center">
            No price history yet — run a scrape job for a group containing this hotel.
          </p>
        ) : (
          <ResponsiveContainer width="100%" height={280}>
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
        )}
      </Card>

      {/* Full, unaggregated price history — every individual scrape */}
      <Card className="mt-4">
        <div className="px-5 py-3.5 border-b flex items-center justify-between">
          <h2 className="text-sm font-bold">
            All Price Data{history && ` (${history.total})`}
          </h2>
          <select
            value={historySource}
            onChange={(e) => updateHistoryParams({ source: e.target.value })}
            className="h-8 rounded-[7px] border border-input bg-background px-2 text-xs"
          >
            <option value="">All sources</option>
            {KNOWN_PROVIDERS.map((s) => (
              <option key={s} value={s}>
                {PROVIDER_LABELS[s]}
              </option>
            ))}
          </select>
        </div>

        {historyLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : history && history.rows.length > 0 ? (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Source</TableHead>
                  <TableHead>Room Type</TableHead>
                  <TableHead className="text-right">Price (THB)</TableHead>
                  <TableHead>Meal Plan</TableHead>
                  <TableHead>Cancellation</TableHead>
                  <TableHead>Check-in</TableHead>
                  <TableHead className="text-right">Scraped</TableHead>
                  <TableHead />
                </TableRow>
              </TableHeader>
              <TableBody>
                {history.rows.map((row) => (
                  <TableRow key={row.id}>
                    <TableCell>
                      <Badge variant="info">{PROVIDER_LABELS[row.source as Provider] ?? row.source}</Badge>
                    </TableCell>
                    <TableCell className="text-sm">{row.room_type}</TableCell>
                    <TableCell className="text-right font-semibold">{formatPrice(row.price_thb)}</TableCell>
                    <TableCell className="text-sm text-muted-foreground">{row.meal_plan ?? '—'}</TableCell>
                    <TableCell className="text-sm text-muted-foreground">{row.cancellation ?? '—'}</TableCell>
                    <TableCell className="text-sm text-muted-foreground">{row.checkin_date}</TableCell>
                    <TableCell className="text-right text-xs text-muted-foreground">
                      {new Date(row.scraped_at).toLocaleString()}
                    </TableCell>
                    <TableCell>
                      {row.source_url && (
                        <a
                          href={row.source_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-muted-foreground hover:text-primary"
                        >
                          <ExternalLink className="h-3.5 w-3.5" />
                        </a>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
            <div className="px-5 pb-4">
              <Pagination
                page={historyPage}
                totalPages={historyTotalPages}
                totalItems={history.total}
                pageSize={HISTORY_PAGE_SIZE}
                pageSizeOptions={[HISTORY_PAGE_SIZE]}
                onPageChange={(p) => updateHistoryParams({ historyPage: p }, false)}
                onPageSizeChange={() => {}}
              />
            </div>
          </>
        ) : (
          <p className="text-sm text-muted-foreground py-8 text-center">
            No price data{historySource ? ` from ${PROVIDER_LABELS[historySource as Provider]}` : ''} yet.
          </p>
        )}
      </Card>
    </div>
  );
}
