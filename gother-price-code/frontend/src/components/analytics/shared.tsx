import { ArrowDown, ArrowUp } from 'lucide-react';
import { TableHead } from '@/components/ui/Table';
import { PROVIDER_LABELS, type Provider } from '@/utils/providers';

/** One row of the heatmap: a hotel, its stay, and a price per provider. */
export interface HeatmapRow {
  hotel_id: string;
  hotel_name: string;
  checkin_date: string;
  prices: Record<string, number | null>;
  cheapestSource: string | null;
}

export function providerLabel(source: string) {
  return PROVIDER_LABELS[source as Provider] ?? source;
}

/** Sortable column header — shows direction only on the active column. */
export function SortHeader({
  label,
  sortKey,
  active,
  direction,
  onSort,
  align = 'left',
}: {
  label: string;
  sortKey: string;
  active: string;
  direction: 'asc' | 'desc';
  onSort: (key: string) => void;
  align?: 'left' | 'right';
}) {
  const isActive = active === sortKey;
  return (
    <TableHead className={align === 'right' ? 'text-right' : undefined}>
      <button
        type="button"
        onClick={() => onSort(sortKey)}
        className={`inline-flex items-center gap-1 hover:text-foreground ${
          isActive ? 'text-foreground font-semibold' : ''
        }`}
      >
        {label}
        {isActive &&
          (direction === 'asc' ? (
            <ArrowUp className="h-3 w-3" />
          ) : (
            <ArrowDown className="h-3 w-3" />
          ))}
      </button>
    </TableHead>
  );
}

