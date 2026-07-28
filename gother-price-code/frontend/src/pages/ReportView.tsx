import { useParams, Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import {
  ArrowLeft,
  Download,
  Loader2,
  Calendar,
  Users,
  BedDouble,
  CheckCircle2,
  XCircle,
  TrendingDown,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { PriceComparisonTable } from '@/components/PriceComparisonTable';
import { getScrapeResults, exportResults, downloadBlob } from '@/api/scrapeJobs';
import { formatDate, formatPrice } from '@/utils/format';

export function ReportView() {
  const { id } = useParams<{ id: string }>();

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

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="container mx-auto py-8 px-4">
        <div className="text-center py-12">
          <h2 className="text-xl font-semibold mb-2">Report not found</h2>
          <Button asChild>
            <Link to="/">Go back to Dashboard</Link>
          </Button>
        </div>
      </div>
    );
  }

  const { job, summary, results } = data;

  return (
    <div className="container mx-auto py-8 px-4">
      {/* Header */}
      <div className="mb-6">
        <Button variant="ghost" asChild className="mb-4">
          <Link to="/">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back to Dashboard
          </Link>
        </Button>
        <div className="flex items-start justify-between">
          <div>
            <h1 className="text-3xl font-bold">Price Comparison Report</h1>
            <div className="flex items-center gap-4 mt-2 text-muted-foreground">
              <div className="flex items-center gap-1">
                <Calendar className="h-4 w-4" />
                <span>
                  {formatDate(job.checkin_date)} - {formatDate(job.checkout_date)}
                </span>
              </div>
              <div className="flex items-center gap-1">
                <BedDouble className="h-4 w-4" />
                <span>{job.rooms} room{job.rooms !== 1 ? 's' : ''}</span>
              </div>
              <div className="flex items-center gap-1">
                <Users className="h-4 w-4" />
                <span>{job.adults} adult{job.adults !== 1 ? 's' : ''}</span>
              </div>
            </div>
          </div>
          <Button onClick={handleExport}>
            <Download className="h-4 w-4 mr-2" />
            Export Excel
          </Button>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-4 mb-6">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-blue-100 rounded-lg">
                <BedDouble className="h-5 w-5 text-blue-600" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Total Hotels</p>
                <p className="text-2xl font-bold">{summary.total_hotels}</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-green-100 rounded-lg">
                <CheckCircle2 className="h-5 w-5 text-green-600" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Successful</p>
                <p className="text-2xl font-bold text-green-600">
                  {summary.successful}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-red-100 rounded-lg">
                <XCircle className="h-5 w-5 text-red-600" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Failed</p>
                <p className="text-2xl font-bold text-red-600">
                  {summary.failed}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-purple-100 rounded-lg">
                <TrendingDown className="h-5 w-5 text-purple-600" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Avg Best Price</p>
                <p className="text-2xl font-bold">
                  {summary.avg_best_price
                    ? formatPrice(summary.avg_best_price)
                    : 'N/A'}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Price Comparison Table */}
      <div className="mb-4">
        <h2 className="text-xl font-semibold mb-4">Price Comparison</h2>
      </div>
      <PriceComparisonTable results={results} />
    </div>
  );
}
