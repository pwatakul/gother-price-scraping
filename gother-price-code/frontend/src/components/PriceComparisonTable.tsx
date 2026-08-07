import { Fragment, useState, useMemo } from 'react';
import { ArrowUpDown, ChevronDown, ChevronRight, CheckCircle2, XCircle, Loader2 } from 'lucide-react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { Button } from '@/components/ui/Button';
import { EvidencePanel } from './EvidencePanel';
import { PriceBadge } from './PriceBadge';
import { GapPill } from './GapPill';
import { cn, formatPrice } from '@/utils';
import { PROVIDER_LABELS, visibleProviders, type Provider } from '@/utils/providers';
import type { HotelPriceComparison, HotelScrapeStatus } from '@/types';

interface PriceComparisonTableProps {
  results: HotelPriceComparison[];
}

type SortKey = 'name' | 'best_price' | 'gother_price' | 'difference';
type SortOrder = 'asc' | 'desc';

export function PriceComparisonTable({ results }: PriceComparisonTableProps) {
  const [sortKey, setSortKey] = useState<SortKey>('name');
  const [sortOrder, setSortOrder] = useState<SortOrder>('asc');
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const sortedResults = useMemo(() => {
    return [...results].sort((a, b) => {
      let comparison = 0;

      switch (sortKey) {
        case 'name':
          comparison = a.hotel.name.localeCompare(b.hotel.name);
          break;
        case 'best_price':
          comparison = (a.best_price ?? Infinity) - (b.best_price ?? Infinity);
          break;
        case 'gother_price':
          comparison = (a.gother_price ?? Infinity) - (b.gother_price ?? Infinity);
          break;
        case 'difference':
          comparison =
            (a.price_difference ?? Infinity) - (b.price_difference ?? Infinity);
          break;
      }

      return sortOrder === 'asc' ? comparison : -comparison;
    });
  }, [results, sortKey, sortOrder]);

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortKey(key);
      setSortOrder('asc');
    }
  };

  const toggleExpanded = (hotelId: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(hotelId)) {
        next.delete(hotelId);
      } else {
        next.add(hotelId);
      }
      return next;
    });
  };

  const getStatusIcon = (status: HotelScrapeStatus) => {
    switch (status) {
      case 'success':
        return <CheckCircle2 className="h-4 w-4 text-green-600" />;
      case 'failed':
        return <XCircle className="h-4 w-4 text-red-600" />;
      case 'processing':
        return <Loader2 className="h-4 w-4 text-blue-600 animate-spin" />;
      default:
        return <div className="h-4 w-4 rounded-full border-2 border-gray-300" />;
    }
  };

  const getPriceForSource = (result: HotelPriceComparison, source: string) => {
    return result.prices.find((p) => p.source === source);
  };

  const SortButton = ({
    label,
    sortKeyValue,
  }: {
    label: string;
    sortKeyValue: SortKey;
  }) => (
    <Button
      variant="ghost"
      size="sm"
      className="-ml-3 h-8"
      onClick={() => handleSort(sortKeyValue)}
    >
      {label}
      <ArrowUpDown className="ml-2 h-4 w-4" />
    </Button>
  );

  // Show the union of providers relevant to any hotel in this result set
  // (Wink only appears for domestic hotels) so the header stays stable.
  const providerColumns: Provider[] = useMemo(() => {
    const anyDomestic = results.some((r) => visibleProviders(r.hotel.country).includes('wink'));
    return anyDomestic ? ['gother', 'agoda', 'trip', 'wink'] : ['gother', 'agoda', 'trip'];
  }, [results]);

  return (
    <div className="border rounded-lg overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50">
            <TableHead className="w-[30px]" />
            <TableHead className="w-[250px]">
              <SortButton label="Hotel" sortKeyValue="name" />
            </TableHead>
            <TableHead className="w-[80px] text-center">Status</TableHead>
            {providerColumns.map((source) => (
              <TableHead key={source} className="text-right">
                {PROVIDER_LABELS[source]}
              </TableHead>
            ))}
            <TableHead className="text-right">
              <SortButton label="Best" sortKeyValue="best_price" />
            </TableHead>
            <TableHead className="text-right">
              <SortButton label="Gap" sortKeyValue="difference" />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {sortedResults.map((result) => {
            const isExpanded = expanded.has(result.hotel.id);
            const rowProviders = visibleProviders(result.hotel.country);

            return (
              <Fragment key={result.hotel.id}>
                <TableRow>
                  <TableCell>
                    <button
                      onClick={() => toggleExpanded(result.hotel.id)}
                      className="text-muted-foreground hover:text-foreground"
                      aria-label={isExpanded ? 'Collapse' : 'Expand'}
                    >
                      {isExpanded ? (
                        <ChevronDown className="h-4 w-4" />
                      ) : (
                        <ChevronRight className="h-4 w-4" />
                      )}
                    </button>
                  </TableCell>

                  <TableCell>
                    <div>
                      <div className="font-medium">{result.hotel.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {result.hotel.city}, {result.hotel.country}
                        {result.hotel.hid && ` · HID ${result.hotel.hid}`}
                      </div>
                    </div>
                  </TableCell>

                  <TableCell className="text-center">
                    {getStatusIcon(result.status)}
                  </TableCell>

                  {providerColumns.map((source) => {
                    const price = getPriceForSource(result, source);
                    const isBest = result.best_source === source;
                    const isApplicable = rowProviders.includes(source);

                    return (
                      <TableCell
                        key={source}
                        className={cn(
                          'text-right',
                          isBest && 'bg-green-50 font-semibold text-green-700'
                        )}
                      >
                        {!isApplicable ? (
                          <span className="text-muted-foreground">n/a</span>
                        ) : price ? (
                          <div className="flex items-center justify-end">
                            <span>{formatPrice(price.price_thb)}</span>
                            <PriceBadge entry={price} />
                          </div>
                        ) : (
                          // Blank (never 0) when no rate exists — REQ-001 F-027
                          <span className="text-muted-foreground">—</span>
                        )}
                      </TableCell>
                    );
                  })}

                  <TableCell className="text-right">
                    {result.best_price ? (
                      <div>
                        <div className="font-semibold text-green-700">
                          {formatPrice(result.best_price)}
                        </div>
                        {result.best_source && (
                          <div className="text-xs text-muted-foreground">
                            via {PROVIDER_LABELS[result.best_source as Provider] ?? result.best_source}
                          </div>
                        )}
                      </div>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </TableCell>

                  <TableCell className="text-right">
                    <GapPill
                      isCheapest={result.best_source === 'gother'}
                      priceDifference={result.price_difference}
                      priceDiffPercent={result.price_diff_percent}
                    />
                  </TableCell>
                </TableRow>

                {isExpanded && (
                  <TableRow>
                    <TableCell colSpan={providerColumns.length + 5} className="p-0">
                      <EvidencePanel result={result} />
                    </TableCell>
                  </TableRow>
                )}
              </Fragment>
            );
          })}
        </TableBody>
      </Table>
    </div>
  );
}
