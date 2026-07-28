import { useState, useMemo } from 'react';
import { ArrowUpDown, ExternalLink, CheckCircle2, XCircle, Loader2 } from 'lucide-react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { cn, formatPrice } from '@/utils';
import type { HotelPriceComparison, HotelScrapeStatus } from '@/types';

interface PriceComparisonTableProps {
  results: HotelPriceComparison[];
}

type SortKey = 'name' | 'best_price' | 'gother_price' | 'difference';
type SortOrder = 'asc' | 'desc';

const OTA_SOURCES = ['gother', 'agoda', 'booking', 'trip.com', 'official'];

export function PriceComparisonTable({ results }: PriceComparisonTableProps) {
  const [sortKey, setSortKey] = useState<SortKey>('name');
  const [sortOrder, setSortOrder] = useState<SortOrder>('asc');

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

  return (
    <div className="border rounded-lg overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50">
            <TableHead className="w-[250px]">
              <SortButton label="Hotel" sortKeyValue="name" />
            </TableHead>
            <TableHead className="w-[80px] text-center">Status</TableHead>
            {OTA_SOURCES.map((source) => (
              <TableHead key={source} className="text-right">
                {source.charAt(0).toUpperCase() + source.slice(1)}
              </TableHead>
            ))}
            <TableHead className="text-right">
              <SortButton label="Best" sortKeyValue="best_price" />
            </TableHead>
            <TableHead className="text-right">
              <SortButton label="vs Gother" sortKeyValue="difference" />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {sortedResults.map((result) => (
            <TableRow key={result.hotel.id}>
              {/* Hotel Name */}
              <TableCell>
                <div>
                  <div className="font-medium">{result.hotel.name}</div>
                  <div className="text-sm text-muted-foreground">
                    {result.hotel.city}, {result.hotel.country}
                  </div>
                </div>
              </TableCell>

              {/* Status */}
              <TableCell className="text-center">
                {getStatusIcon(result.status)}
              </TableCell>

              {/* OTA Prices */}
              {OTA_SOURCES.map((source) => {
                const price = getPriceForSource(result, source);
                const isBest = result.best_source === source;

                return (
                  <TableCell
                    key={source}
                    className={cn(
                      'text-right',
                      isBest && 'bg-green-50 font-semibold text-green-700'
                    )}
                  >
                    {price ? (
                      <div className="flex items-center justify-end gap-1">
                        <span>{formatPrice(price.price_thb)}</span>
                        {price.source_url && (
                          <a
                            href={price.source_url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-muted-foreground hover:text-primary"
                          >
                            <ExternalLink className="h-3 w-3" />
                          </a>
                        )}
                      </div>
                    ) : (
                      <span className="text-muted-foreground">-</span>
                    )}
                  </TableCell>
                );
              })}

              {/* Best Price */}
              <TableCell className="text-right">
                {result.best_price ? (
                  <div>
                    <div className="font-semibold text-green-700">
                      {formatPrice(result.best_price)}
                    </div>
                    {result.best_source && (
                      <div className="text-xs text-muted-foreground">
                        via {result.best_source}
                      </div>
                    )}
                  </div>
                ) : (
                  <span className="text-muted-foreground">-</span>
                )}
              </TableCell>

              {/* Difference vs Gother */}
              <TableCell className="text-right">
                {result.price_difference !== null ? (
                  <Badge
                    variant={result.price_difference > 0 ? 'error' : 'success'}
                  >
                    {result.price_difference > 0 ? '+' : ''}
                    {formatPrice(result.price_difference)}
                  </Badge>
                ) : (
                  <span className="text-muted-foreground">-</span>
                )}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
