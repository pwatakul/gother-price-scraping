import { Link, useSearchParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Search, Download, Loader2 } from 'lucide-react';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Badge } from '@/components/ui/Badge';
import { Pagination } from '@/components/Pagination';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { listHotels, listCountries, listCities } from '@/api/hotelDirectory';
import apiClient from '@/api/client';
import { downloadBlob } from '@/api/scrapeJobs';
import { formatPrice, formatRelativeTime } from '@/utils/format';

const DEFAULT_PAGE_SIZE = 25;

/// Filters + page + pageSize all live in the URL (?country=&city=&q=&page=&pageSize=)
/// so refresh/back-button/shared links preserve the exact view — REQ-007 F-007.
export function HotelsList() {
  const [searchParams, setSearchParams] = useSearchParams();

  const country = searchParams.get('country') ?? '';
  const city = searchParams.get('city') ?? '';
  const search = searchParams.get('q') ?? '';
  const page = Number(searchParams.get('page') ?? '0');
  const pageSize = Number(searchParams.get('pageSize') ?? DEFAULT_PAGE_SIZE);

  /** Updates one or more params; any change other than `page` itself
   * resets page to 0 so a filter change never leaves you on an
   * out-of-range page showing no results. */
  const updateParams = (updates: Record<string, string | number | undefined>, resetPage = true) => {
    const next = new URLSearchParams(searchParams);
    for (const [key, value] of Object.entries(updates)) {
      if (value === undefined || value === '') {
        next.delete(key);
      } else {
        next.set(key, String(value));
      }
    }
    if (resetPage) {
      next.delete('page');
    }
    setSearchParams(next);
  };

  const { data: countries } = useQuery({ queryKey: ['hotels', 'countries'], queryFn: listCountries });
  const { data: cities } = useQuery({
    queryKey: ['hotels', 'cities', country],
    queryFn: () => listCities(country || undefined),
  });

  const { data, isLoading } = useQuery({
    queryKey: ['hotels', 'list', country, city, search, page, pageSize],
    queryFn: () =>
      listHotels({
        country: country || undefined,
        city: city || undefined,
        q: search || undefined,
        limit: pageSize,
        offset: page * pageSize,
      }),
  });

  const handleExport = async () => {
    const response = await apiClient.get('/hotels/export', {
      params: { country: country || undefined, city: city || undefined, q: search || undefined },
      responseType: 'blob',
    });
    downloadBlob(response.data, 'hotels.csv');
  };

  const totalPages = data ? Math.ceil(data.total / pageSize) : 0;

  return (
    <div className="max-w-[1400px] mx-auto py-6 px-7">
      <div className="flex items-start justify-between mb-6">
        <div>
          <h1 className="text-xl font-bold">🏨 All Hotels</h1>
          <p className="text-sm text-muted-foreground mt-1">
            Every hotel tracked in the system, across all groups.
            {data && ` ${data.total} total.`}
          </p>
        </div>
        <Button onClick={handleExport}>
          <Download className="h-4 w-4 mr-2" />
          Export List
        </Button>
      </div>

      <div className="flex items-center gap-2.5 mb-4">
        <div className="relative w-[280px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => updateParams({ q: e.target.value })}
            placeholder="Search hotels..."
            className="pl-9"
          />
        </div>
        <select
          value={country}
          onChange={(e) => updateParams({ country: e.target.value, city: undefined })}
          className="h-10 rounded-[7px] border border-input bg-background px-3 text-sm w-[160px]"
        >
          <option value="">All countries</option>
          {countries?.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>
        <select
          value={city}
          onChange={(e) => updateParams({ city: e.target.value })}
          className="h-10 rounded-[7px] border border-input bg-background px-3 text-sm w-[160px]"
        >
          <option value="">All cities</option>
          {cities?.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>
      </div>

      <Card>
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : data && data.hotels.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Hotel Name</TableHead>
                <TableHead>City</TableHead>
                <TableHead>Country</TableHead>
                <TableHead>Groups</TableHead>
                <TableHead className="text-right">Last Price</TableHead>
                <TableHead className="text-right">Scraped</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.hotels.map((h) => (
                <TableRow key={h.id}>
                  <TableCell className="font-medium">
                    <Link to={`/hotels/${h.id}`} className="hover:text-brand-600">
                      {h.name}
                    </Link>
                    {h.hid && <div className="text-xs text-muted-foreground">HID {h.hid}</div>}
                  </TableCell>
                  <TableCell>{h.city || <span className="text-muted-foreground">—</span>}</TableCell>
                  <TableCell>{h.country}</TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {h.group_names.slice(0, 2).map((g) => (
                        <Badge key={g} variant="info">
                          {g}
                        </Badge>
                      ))}
                      {h.group_names.length > 2 && (
                        <span className="text-xs text-muted-foreground">+{h.group_names.length - 2}</span>
                      )}
                    </div>
                  </TableCell>
                  <TableCell className="text-right">
                    {h.last_price_thb ? (
                      <span className="font-semibold">{formatPrice(h.last_price_thb)}</span>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    {h.last_scraped_at ? formatRelativeTime(h.last_scraped_at) : '—'}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <div className="text-center py-12 text-muted-foreground">No hotels match these filters.</div>
        )}
      </Card>

      {data && (
        <Pagination
          page={page}
          totalPages={totalPages}
          totalItems={data.total}
          pageSize={pageSize}
          onPageChange={(p) => updateParams({ page: p }, false)}
          onPageSizeChange={(size) => updateParams({ pageSize: size })}
        />
      )}
    </div>
  );
}
