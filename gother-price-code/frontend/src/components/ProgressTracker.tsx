import { useEffect } from 'react';
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
import type { ScrapeJobWithProgress } from '@/types';
import { formatPercentage } from '@/utils/format';

interface ProgressTrackerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  job: ScrapeJobWithProgress | null;
  onCancel: () => void;
  onViewResults: () => void;
  isLoading?: boolean;
}

export function ProgressTracker({
  open,
  onOpenChange,
  job,
  onCancel,
  onViewResults,
  isLoading,
}: ProgressTrackerProps) {
  if (!job) return null;

  const { progress, status } = job;
  const total = progress.total;
  const completed = progress.completed + progress.failed;
  const percentage = total > 0 ? (completed / total) * 100 : 0;

  const isComplete = status === 'completed';
  const isFailed = status === 'failed';
  const isCancelled = status === 'cancelled';
  const isRunning = status === 'processing' || status === 'pending';

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {isComplete
              ? 'Scraping Complete'
              : isFailed
              ? 'Scraping Failed'
              : isCancelled
              ? 'Scraping Cancelled'
              : 'Scraping in Progress'}
          </DialogTitle>
          <DialogDescription>
            {isComplete
              ? 'Price comparison is ready to view'
              : isRunning
              ? 'Fetching prices from multiple sources...'
              : 'The job has been stopped'}
          </DialogDescription>
        </DialogHeader>

        <div className="py-4 space-y-4">
          {/* Progress Bar */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Progress</span>
              <span>{formatPercentage(percentage)}</span>
            </div>
            <Progress value={percentage} className="h-3" />
          </div>

          {/* Stats */}
          <div className="grid grid-cols-3 gap-4 text-center">
            <div className="p-3 bg-muted rounded-lg">
              <div className="text-2xl font-bold text-green-600">
                {progress.completed}
              </div>
              <div className="text-xs text-muted-foreground">Success</div>
            </div>
            <div className="p-3 bg-muted rounded-lg">
              <div className="text-2xl font-bold text-red-600">
                {progress.failed}
              </div>
              <div className="text-xs text-muted-foreground">Failed</div>
            </div>
            <div className="p-3 bg-muted rounded-lg">
              <div className="text-2xl font-bold text-yellow-600">
                {progress.pending}
              </div>
              <div className="text-xs text-muted-foreground">Pending</div>
            </div>
          </div>

          {/* Status Indicator */}
          <div className="flex items-center justify-center gap-2 text-sm">
            {isRunning && (
              <>
                <Loader2 className="h-4 w-4 animate-spin text-blue-600" />
                <span className="text-blue-600">Processing...</span>
              </>
            )}
            {isComplete && (
              <>
                <CheckCircle2 className="h-4 w-4 text-green-600" />
                <span className="text-green-600">Completed</span>
              </>
            )}
            {isFailed && (
              <>
                <XCircle className="h-4 w-4 text-red-600" />
                <span className="text-red-600">Failed</span>
              </>
            )}
            {isCancelled && (
              <>
                <Circle className="h-4 w-4 text-gray-600" />
                <span className="text-gray-600">Cancelled</span>
              </>
            )}
          </div>
        </div>

        <DialogFooter>
          {isRunning ? (
            <Button
              variant="destructive"
              onClick={onCancel}
              disabled={isLoading}
            >
              Cancel Job
            </Button>
          ) : (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Close
              </Button>
              {(isComplete || progress.completed > 0) && (
                <Button onClick={onViewResults}>View Results</Button>
              )}
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
