import { cn, formatPrice } from '@/utils';

interface GapPillProps {
  /** true when Gother itself is the best/cheapest source for this hotel */
  isCheapest: boolean;
  priceDifference: number | null;
  priceDiffPercent: number | null;
}

/** Combined THB+% gap pill matching the mockup's report-table gap cell:
 * green "🟢 Cheapest" when Gother wins, red "🔴 +฿X / +Y%" when it loses. */
export function GapPill({ isCheapest, priceDifference, priceDiffPercent }: GapPillProps) {
  if (isCheapest) {
    return (
      <span className="inline-flex items-center rounded-full bg-green-100 px-2.5 py-0.5 text-xs font-semibold text-green-700">
        🟢 Cheapest
      </span>
    );
  }

  if (priceDifference === null || priceDiffPercent === null) {
    return <span className="text-muted-foreground">—</span>;
  }

  return (
    <span
      className={cn(
        'inline-flex flex-col items-end rounded-full px-2.5 py-0.5 text-xs font-semibold',
        priceDifference > 0 ? 'bg-red-100 text-red-600' : 'bg-green-100 text-green-700'
      )}
    >
      <span>
        {priceDifference > 0 ? '🔴' : '🟢'} {priceDifference > 0 ? '+' : ''}
        {formatPrice(priceDifference)}
      </span>
      <span className="text-[10px] font-normal opacity-80">
        {priceDiffPercent > 0 ? '+' : ''}
        {priceDiffPercent.toFixed(1)}%
      </span>
    </span>
  );
}
