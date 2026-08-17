import { CheckCircle2, XCircle, Loader2, Circle } from 'lucide-react';
import { Progress } from '@/components/ui/Progress';
import { Button } from '@/components/ui/Button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/Dialog';
import type { HotelPriceComparison, HotelScrapeStatus, ScrapeResultsResponse } from '@/types';
import { formatPercentage, formatPrice } from '@/utils/format';

interface ProgressTrackerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  results: ScrapeResultsResponse | null;
  onCancel: () => void;
  onViewResults: () => void;
  isLoading?: boolean;
}

function statusIcon(status: HotelScrapeStatus) {
  switch (status) {
    case 'success':
      return <CheckCircle2 className="h-4 w-4 text-green-600 shrink-0" />;
    case 'failed':
      return <XCircle className="h-4 w-4 text-red-600 shrink-0" />;
    case 'processing':
      return <Loader2 className="h-4 w-4 text-brand-600 animate-spin shrink-0" />;
    default:
      return <Circle className="h-4 w-4 text-gray-300 shrink-0" />;
  }
}

export function ProgressTracker({
  open,
  onOpenChange,
  results,
  onCancel,
  onViewResults,
  isLoading,
}: ProgressTrackerProps) {
  if (!results) return null;

  const { job, results: hotelResults } = results;

  const counts = hotelResults.reduce(
    (acc, r) => {
      if (r.status === 'success') acc.completed += 1;
      else if (r.status === 'failed') acc.failed += 1;
      else acc.pending += 1;
      return acc;
    },
    { completed: 0, failed: 0, pending: 0 }
  );
  const total = hotelResults.length;
  const done = counts.completed + counts.failed;
  const percentage = total > 0 ? (done / total) * 100 : 0;

  const isComplete = job.status === 'completed';
  const isFailed = job.status === 'failed';
  const isCancelled = job.status === 'cancelled';
  const isRunning = job.status === 'processing' || job.status === 'pending';

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>
            {isComplete
              ? '✅ Scraping Complete'
              : isFailed
              ? '❌ Scraping Failed'
              : isCancelled
              ? 'Scraping Cancelled'
              : '⏳ Scraping in Progress'}
          </DialogTitle>
          <DialogDescription>
            {isComplete
              ? 'Price comparison is ready to view'
              : isRunning
              ? 'Fetching prices from multiple sources... polling every 5 seconds'
              : 'The job has been stopped'}
          </DialogDescription>
        </DialogHeader>

        <div className="py-2 space-y-4">
          {/* Progress Bar */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Overall Progress</span>
              <span>{formatPercentage(percentage)}</span>
            </div>
            <Progress value={percentage} className="h-3" />
          </div>

          {/* Stats */}
          <div className="grid grid-cols-3 gap-4 text-center">
            <div className="p-3 bg-muted rounded-lg">
              <div className="text-2xl font-bold text-green-600">{counts.completed}</div>
              <div className="text-xs text-muted-foreground">Success</div>
            </div>
            <div className="p-3 bg-muted rounded-lg">
              <div className="text-2xl font-bold text-red-600">{counts.failed}</div>
              <div className="text-xs text-muted-foreground">Failed</div>
            </div>
            <div className="p-3 bg-muted rounded-lg">
              <div className="text-2xl font-bold text-amber-600">{counts.pending}</div>
              <div className="text-xs text-muted-foreground">Pending</div>
            </div>
          </div>

          {/* Per-hotel status log */}
          <div className="border rounded-lg max-h-64 overflow-y-auto divide-y">
            {hotelResults.map((r: HotelPriceComparison) => (
              <div key={r.hotel.id} className="flex items-center gap-2 px-3 py-2 text-sm">
                {statusIcon(r.status)}
                <span className="flex-1 min-w-0 truncate">{r.hotel.name}</span>
                {r.status === 'processing' && (
                  <span className="text-xs text-amber-600">Searching...</span>
                )}
                {r.status === 'failed' && r.error_message && (
                  <span className="text-xs text-red-600 truncate max-w-[160px]">
                    {r.error_message}
                  </span>
                )}
                {r.status === 'success' && r.best_price != null && (
                  <span className="text-xs font-semibold text-green-700">
                    {formatPrice(r.best_price)}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>

        <DialogFooter>
          {isRunning ? (
            <Button variant="destructive" onClick={onCancel} disabled={isLoading}>
              Cancel Job
            </Button>
          ) : (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Close
              </Button>
              {(isComplete || counts.completed > 0) && (
                <Button onClick={onViewResults}>View Report →</Button>
              )}
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
