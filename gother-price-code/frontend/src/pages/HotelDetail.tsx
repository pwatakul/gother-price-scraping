import { useMemo, useState } from 'react';
import { useParams, Link, useSearchParams } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, Download, ExternalLink, Loader2, Pencil } from 'lucide-react';
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
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/Dialog';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { Pagination } from '@/components/Pagination';
import { getHotelDetail, updateHotel } from '@/api/hotelDirectory';
import {
  getBookingWindow,
  getHotelTrend,
  getTrendWindows,
  listPriceHistory,
} from '@/api/analytics';
import apiClient from '@/api/client';
import { downloadBlob } from '@/api/scrapeJobs';
import { formatPrice } from '@/utils/format';
import { KNOWN_PROVIDERS, PROVIDER_LABELS, type Provider } from '@/utils/providers';
import type { HotelPriceHistoryRow } from '@/types';

const HISTORY_PAGE_SIZE = 20;

/** REQ-008-v1.1 F-001 — the windows the scheduler is required to produce.
 * Mirrors STANDARD_BOOKING_WINDOWS in the backend scheduler; kept in sync
 * by hand so a missing window is visible as a gap. */
const STANDARD_BOOKING_WINDOWS = [1, 3, 7, 14, 30];
/** Rows scanned to build the coverage grid — 5 windows × several sources
 * per scheduler fire, so this spans several days of scheduled runs. */
const COVERAGE_SAMPLE_SIZE = 500;

const MS_PER_DAY = 86_400_000;

/** How the price was obtained. `serpapi`/`gother` are real scrapes;
 * `gemini` is an AI estimate used only where scraping found nothing, so it
 * is deliberately styled as a warning and never blends in (ADR-011). */
function MethodBadge({ viaMethod }: { viaMethod: string }) {
  if (viaMethod === 'gemini') {
    return (
      <Badge variant="warning" title="AI estimate — no scraped price was available">
        Gemini (AI)
      </Badge>
    );
  }
  if (viaMethod === 'mock') {
    return <Badge variant="error">Mock (fake)</Badge>;
  }
  return (
    <Badge variant="secondary" title="Scraped price">
      {viaMethod === 'serpapi' ? 'SerpAPI' : viaMethod || '—'}
    </Badge>
  );
}

function bookingWindowOf(row: HotelPriceHistoryRow): number {
  const checkin = Date.parse(`${row.checkin_date}T00:00:00Z`);
  const scrapedDay = Date.parse(`${row.scraped_at.slice(0, 10)}T00:00:00Z`);
  return Math.round((checkin - scrapedDay) / MS_PER_DAY);
}

/**
 * Series colours, validated with the dataviz palette checker (light mode):
 * all inside the lightness band, above the chroma floor, worst adjacent CVD
 * ΔE 9.1 and normal-vision ΔE 19.6 — both above the required floors.
 *
 * Gother carries the brand red; the OTAs follow the reference categorical
 * slot order. The previous palette failed validation: Hotel Direct was
 * near-black (#111827), below the chroma floor, so it read as grey rather
 * than as a hue.
 *
 * Three of these sit under 3:1 against a white surface, which the checker
 * flags as needing relief — satisfied here by the always-present legend and
 * the price table below the chart.
 */
const SOURCE_COLORS: Record<string, string> = {
  gother: '#bc2c1c', // brand
  agoda: '#2a78d6',
  trip: '#eb6834',
  booking: '#1baf7a',
  expedia: '#eda100',
  priceline: '#e87ba4',
  klook: '#008300',
  traveloka: '#4a3aa7',
  direct: '#e34948',
  wink: '#8b5cf6',
};

export function HotelDetail() {
  const { id } = useParams<{ id: string }>();
  const [searchParams, setSearchParams] = useSearchParams();
  const historySource = searchParams.get('source') ?? '';
  const historyPage = Number(searchParams.get('historyPage') ?? '0');

  const queryClient = useQueryClient();
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [editName, setEditName] = useState('');
  const [editCity, setEditCity] = useState('');
  const [editCountry, setEditCountry] = useState('');
  const [editError, setEditError] = useState<string | null>(null);

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

  const { data: coverage, isLoading: coverageLoading } = useQuery({
    queryKey: ['price-history', id, 'booking-window-coverage'],
    queryFn: () => listPriceHistory({ hotelId: id!, limit: COVERAGE_SAMPLE_SIZE, offset: 0 }),
    enabled: !!id,
  });

  // One cell per standard booking window, holding the CHEAPEST quote for the
  // most recent stay observed in that window. A null cell means the grid has
  // a hole.
  //
  // Selection is two steps, and the order matters:
  //   1. Pick the latest check-in date seen for the window. For a fixed
  //      window, check-in date moves with the scrape date, so this also
  //      selects the most recent scrape — one filter does both.
  //   2. Among rows for that stay, take the minimum price.
  //
  // Step 1 first, because a minimum taken across check-in dates would compare
  // different stays and report the cheapest *date*, not the cheapest provider
  // (ADR-013 — the comparison unit is hotel + check-in date).
  const coverageRows = useMemo(() => {
    const byWindow = new Map<number, HotelPriceHistoryRow[]>();
    for (const row of coverage?.rows ?? []) {
      const key = bookingWindowOf(row);
      const bucket = byWindow.get(key);
      if (bucket) bucket.push(row);
      else byWindow.set(key, [row]);
    }

    return STANDARD_BOOKING_WINDOWS.map((window) => {
      const rows = byWindow.get(window) ?? [];
      if (rows.length === 0) return { window, row: null, quoteCount: 0 };

      const latestStay = rows.reduce(
        (latest, r) => (r.checkin_date > latest ? r.checkin_date : latest),
        rows[0].checkin_date
      );
      const forStay = rows.filter((r) => r.checkin_date === latestStay);

      // Cheapest wins; a tie goes to the more recently scraped row so the
      // Scraped column shows the freshest confirmation of that price.
      const cheapest = forStay.reduce((best, r) =>
        r.price_thb < best.price_thb ||
        (r.price_thb === best.price_thb && r.scraped_at > best.scraped_at)
          ? r
          : best
      );

      // How many providers this minimum was chosen from — a "lowest" out of
      // one is not a comparison, and the table shouldn't imply it was.
      const providers = new Set(forStay.map((r) => r.source));
      return { window, row: cheapest, quoteCount: providers.size };
    });
  }, [coverage]);

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

  // Booking window drives the chart: prices differ enormously by how far
  // ahead you book, so every series must sit on the same window or the
  // comparison is meaningless (ADR-013). Options come from the data.
  const { data: windows } = useQuery({
    queryKey: ['trend-windows', id],
    queryFn: () => getTrendWindows(id!),
    enabled: !!id,
  });
  const defaultWindow = windows?.[0]?.days_in_advance;
  const windowParam = searchParams.get('window');
  const activeWindow = windowParam != null ? Number(windowParam) : defaultWindow;

  const { data: windowTrend } = useQuery({
    queryKey: ['trend', id, activeWindow],
    queryFn: () => getHotelTrend(id!, 90, undefined, activeWindow),
    enabled: !!id && activeWindow != null,
  });

  // Price against how far ahead the stay was booked. Inherently
  // like-for-like: days-in-advance *is* the x-axis, so every series sits
  // on the same window at each point (ADR-013).
  const { data: bookingWindow } = useQuery({
    queryKey: ['booking-window', id],
    queryFn: () => getBookingWindow(id!),
    enabled: !!id,
  });

  const bookingWindowChartData = useMemo(() => {
    if (!bookingWindow) return [];
    const byDays = new Map<number, Record<string, number | string>>();
    bookingWindow.forEach((point) => {
      const row = byDays.get(point.days_in_advance) ?? { days_in_advance: point.days_in_advance };
      row[point.source] = point.avg_price_thb;
      byDays.set(point.days_in_advance, row);
    });
    // Far-out bookings on the left, last-minute on the right — reading
    // left to right follows the approach of the stay.
    return Array.from(byDays.values()).sort(
      (a, b) => Number(b.days_in_advance) - Number(a.days_in_advance)
    );
  }, [bookingWindow]);

  const editMutation = useMutation({
    mutationFn: () =>
      updateHotel(id!, { name: editName, city: editCity, country: editCountry }),
    onSuccess: () => {
      setIsEditOpen(false);
      setEditError(null);
      // The name feeds the scraper query and several caches keyed by hotel,
      // so refresh everything rather than patching one entry.
      queryClient.invalidateQueries({ queryKey: ['hotels'] });
      queryClient.invalidateQueries({ queryKey: ['analytics'] });
    },
    onError: (err: unknown) => {
      const message =
        (err as { response?: { data?: { error?: { message?: string } } } })?.response?.data?.error
          ?.message ?? 'Could not save those details.';
      setEditError(message);
    },
  });

  const openEdit = () => {
    if (!data) return;
    setEditName(data.hotel.name);
    setEditCity(data.hotel.city);
    setEditCountry(data.hotel.country);
    setEditError(null);
    setIsEditOpen(true);
  };

  const trendChartData = useMemo(() => {
    if (!windowTrend) return [];
    const byDay = new Map<string, Record<string, number | string>>();
    windowTrend.forEach((point) => {
      const day = point.day.slice(0, 10);
      const row = byDay.get(day) ?? { day };
      row[point.source] = point.avg_price_thb;
      byDay.set(day, row);
    });
    return Array.from(byDay.values()).sort((a, b) => String(a.day).localeCompare(String(b.day)));
  }, [windowTrend]);

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

  const { hotel, group_names } = data;

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
        <div className="flex items-center gap-2">
        <Button variant="outline" onClick={openEdit}>
          <Pencil className="h-4 w-4 mr-2" />
          Edit Hotel
        </Button>
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
      </div>

      {/* Edit dialog */}
      <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
        <DialogContent className="sm:max-w-md">
          <form
            onSubmit={(e) => {
              e.preventDefault();
              editMutation.mutate();
            }}
          >
            <DialogHeader>
              <DialogTitle>Edit Hotel</DialogTitle>
              <DialogDescription>
                These values are what the scraper searches for, so keep the name specific enough
                to identify the property.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="edit-name">Hotel name *</Label>
                <Input
                  id="edit-name"
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  required
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="edit-city">City</Label>
                  <Input
                    id="edit-city"
                    value={editCity}
                    onChange={(e) => setEditCity(e.target.value)}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="edit-country">Country</Label>
                  <Input
                    id="edit-country"
                    value={editCountry}
                    onChange={(e) => setEditCountry(e.target.value)}
                  />
                </div>
              </div>
              <p className="text-xs text-muted-foreground">
                Past prices are kept — they stay attached to this hotel. Future searches use the
                new details.
              </p>
              {editError && <p className="text-xs text-red-600">{editError}</p>}
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setIsEditOpen(false)}>
                Cancel
              </Button>
              <Button type="submit" disabled={!editName.trim() || editMutation.isPending}>
                {editMutation.isPending ? 'Saving...' : 'Save Changes'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Card className="p-5">
        <div className="flex items-center justify-between mb-3">
          <div>
            <h2 className="text-sm font-bold">
              Price Trend (last 90 days)
              {activeWindow != null && (
                <span className="font-normal text-muted-foreground">
                  {' '}
                  — booked {activeWindow} day{activeWindow !== 1 ? 's' : ''} ahead
                </span>
              )}
            </h2>
            <p className="text-xs text-muted-foreground mt-0.5">
              All providers on the same booking window, so the comparison is like for like.
            </p>
          </div>
          {windows && windows.length > 0 && (
            <select
              className="h-8 rounded-md border border-input bg-background px-2 text-xs"
              value={activeWindow ?? ''}
              onChange={(e) => updateHistoryParams({ window: e.target.value }, false)}
            >
              {windows.map((w) => (
                <option key={w.days_in_advance} value={w.days_in_advance}>
                  +{w.days_in_advance}d ({w.sample_count} price{w.sample_count !== 1 ? 's' : ''})
                </option>
              ))}
            </select>
          )}
        </div>
        {trendChartData.length === 0 ? (
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

      {/* Price against lead time — how the market moves as the stay
          approaches. Averaged across scrapes per window. */}
      <Card className="p-5 mt-4">
        <h2 className="text-sm font-bold">Booking Window</h2>
        <p className="text-xs text-muted-foreground mt-0.5 mb-3">
          Average price by how far ahead the stay is booked. Each point compares providers on the
          same window, so the lines are directly comparable.
        </p>
        {bookingWindowChartData.length === 0 ? (
          <p className="text-sm text-muted-foreground py-8 text-center">
            No booking-window data yet — run a price search across several windows.
          </p>
        ) : (
          <ResponsiveContainer width="100%" height={260}>
            <LineChart data={bookingWindowChartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
              <XAxis
                dataKey="days_in_advance"
                tick={{ fontSize: 11 }}
                label={{
                  value: 'Days before check-in',
                  position: 'insideBottom',
                  offset: -5,
                  fontSize: 11,
                }}
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
                  dot={{ r: 2 }}
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        )}
      </Card>

      {/* REQ-008-v1.1 F-008 — the standard booking-window grid. Empty
          cells are shown deliberately: a hole means the mandatory grid
          was not fully collected. */}
      <Card className="mt-4">
        <div className="px-5 py-3.5 border-b">
          <h2 className="text-sm font-bold">Booking Window Coverage</h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            <strong className="font-semibold">Cheapest</strong> quote per booking window, for the
            most recent stay observed — 1 night, 1 room, 2 adults.{' '}
            <span className="text-[#854d0e]">Gemini (AI)</span> rows are estimates, used only where
            scraping returned nothing — treat them as indicative, not measured.
          </p>
        </div>
        {coverageLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Window</TableHead>
                <TableHead>Cheapest source</TableHead>
                <TableHead>Method</TableHead>
                <TableHead className="text-right">Lowest price (THB)</TableHead>
                <TableHead className="text-right">Of</TableHead>
                <TableHead>Check-in</TableHead>
                <TableHead className="text-right">Scraped</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {coverageRows.map(({ window, row, quoteCount }) => (
                <TableRow key={window}>
                  <TableCell className="text-sm font-semibold">+{window}d</TableCell>
                  <TableCell>
                    {row ? (
                      <Badge variant="info">
                        {PROVIDER_LABELS[row.source as Provider] ?? row.source}
                      </Badge>
                    ) : (
                      <span className="text-sm text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  <TableCell>
                    {row ? (
                      <MethodBadge viaMethod={row.via_method} />
                    ) : (
                      <span className="text-sm text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  <TableCell className="text-right font-semibold">
                    {row ? (
                      formatPrice(row.price_thb)
                    ) : (
                      <span className="font-normal text-muted-foreground">No data</span>
                    )}
                  </TableCell>
                  <TableCell className="text-right text-xs text-muted-foreground">
                    {quoteCount > 0 ? (
                      <span
                        title={
                          quoteCount === 1
                            ? 'Only one provider quoted this stay — nothing to be cheapest against'
                            : `Lowest of ${quoteCount} providers quoting this stay`
                        }
                        className={quoteCount === 1 ? 'text-amber-700' : undefined}
                      >
                        {quoteCount} {quoteCount === 1 ? 'provider' : 'providers'}
                      </span>
                    ) : (
                      '—'
                    )}
                  </TableCell>
                  <TableCell className="text-sm text-muted-foreground">
                    {row?.checkin_date ?? '—'}
                  </TableCell>
                  <TableCell className="text-right text-xs text-muted-foreground">
                    {row ? new Date(row.scraped_at).toLocaleString() : '—'}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
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
                  <TableHead>Method</TableHead>
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
                    <TableCell>
                      <MethodBadge viaMethod={row.via_method} />
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
