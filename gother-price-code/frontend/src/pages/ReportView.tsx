import { useMemo, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeft, Download, Loader2, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Input } from '@/components/ui/Input';
import { PriceComparisonTable } from '@/components/PriceComparisonTable';
import { getScrapeResults, exportResults, downloadBlob } from '@/api/scrapeJobs';
import { formatDate, formatPrice, getStatusColor } from '@/utils/format';

type ResultFilter = 'all' | 'winning' | 'losing' | 'not_found';

export function ReportView() {
  const { id } = useParams<{ id: string }>();
  const [search, setSearch] = useState('');
  const [filter, setFilter] = useState<ResultFilter>('all');

  const { data, isLoading, error } = useQuery({
    queryKey: ['scrapeResults', id],
    queryFn: () => getScrapeResults(id!),
    enabled: !!id,
  });

  const handleExport = async () => {
    try {
      const blob = await exportResults(id!);
      downloadBlob(blob, `hotel-price-report-${id}.xlsx`);
    } catch (error) {
      console.error('Failed to export:', error);
    }
  };

  const filteredResults = useMemo(() => {
    if (!data) return [];
    let rows = data.results;

    const q = search.trim().toLowerCase();
    if (q) {
      rows = rows.filter((r) => r.hotel.name.toLowerCase().includes(q));
    }

    if (filter === 'winning') {
      rows = rows.filter((r) => r.best_source === 'gother');
    } else if (filter === 'losing') {
      rows = rows.filter((r) => r.best_source !== null && r.best_source !== 'gother');
    } else if (filter === 'not_found') {
      rows = rows.filter((r) => r.status === 'failed' || r.prices.length === 0);
    }

    return rows;
  }, [data, search, filter]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="max-w-[1400px] mx-auto py-6 px-7">
        <div className="text-center py-12">
          <h2 className="text-xl font-semibold mb-2">Report not found</h2>
          <Button asChild>
            <Link to="/">Go back to Dashboard</Link>
          </Button>
        </div>
      </div>
    );
  }

  const { job, summary } = data;

  return (
    <div className="max-w-[1400px] mx-auto py-6 px-7">
      {/* Back link */}
      <Link
        to="/"
        className="inline-flex items-center gap-1.5 text-sm text-slate-500 hover:text-slate-900 mb-3.5"
      >
        <ArrowLeft className="h-3.5 w-3.5" />
        Back to Dashboard
      </Link>

      {/* Header */}
      <div className="flex items-start justify-between mb-5">
        <div>
          <h1 className="text-xl font-bold">📋 Price Comparison Report</h1>
          <p className="text-sm text-muted-foreground mt-1">
            {formatDate(job.checkin_date)} – {formatDate(job.checkout_date)} · {job.rooms} rm /{' '}
            {job.adults} ad · Method: {job.method} ·{' '}
            <Badge className={getStatusColor(job.status)}>
              {job.status.charAt(0).toUpperCase() + job.status.slice(1)}
            </Badge>
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" asChild>
            <Link to="/">
              <RefreshCw className="h-4 w-4 mr-2" />
              New Search
            </Link>
          </Button>
          <Button onClick={handleExport}>
            <Download className="h-4 w-4 mr-2" />
            Export Excel
          </Button>
        </div>
      </div>

      {/* KPIs */}
      <div className="grid grid-cols-4 gap-3.5 mb-5">
        <Card className="p-[18px_20px]">
          <div className="text-[11px] font-semibold uppercase tracking-wide text-slate-500">
            Hotels Scraped
          </div>
          <div className="text-[26px] font-bold mt-1.5">{summary.total_hotels}</div>
        </Card>
        <Card className="p-[18px_20px]">
          <div className="text-[11px] font-semibold uppercase tracking-wide text-slate-500">
            Successful ✅
          </div>
          <div className="text-[26px] font-bold mt-1.5 text-green-600">{summary.successful}</div>
        </Card>
        <Card className="p-[18px_20px]">
          <div className="text-[11px] font-semibold uppercase tracking-wide text-slate-500">
            Failed ❌
          </div>
          <div className="text-[26px] font-bold mt-1.5 text-red-600">{summary.failed}</div>
        </Card>
        <Card className="p-[18px_20px]">
          <div className="text-[11px] font-semibold uppercase tracking-wide text-slate-500">
            Avg Best Price
          </div>
          <div className="text-[26px] font-bold mt-1.5">
            {summary.avg_best_price ? formatPrice(summary.avg_best_price) : 'N/A'}
          </div>
          <div className="text-[11px] text-slate-400 mt-0.5">across all hotels</div>
        </Card>
      </div>

      {/* Filter row */}
      <div className="flex items-center gap-2.5 mb-4">
        <Input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="🔍 Search hotels..."
          className="w-[280px]"
        />
        <select
          value={filter}
          onChange={(e) => setFilter(e.target.value as ResultFilter)}
          className="h-10 rounded-[7px] border border-input bg-background px-3 text-sm w-[180px]"
        >
          <option value="all">All hotels</option>
          <option value="winning">Gother winning 🟢</option>
          <option value="losing">Gother losing 🔴</option>
          <option value="not_found">Not found ❌</option>
        </select>
        <div className="ml-auto text-xs text-slate-400">
          🟢 = Gother cheapest &nbsp; 🔴 = Gother losing &nbsp; ⚠️ = Not apples-to-apples
        </div>
      </div>

      <PriceComparisonTable results={filteredResults} />
    </div>
  );
}
