import { ExternalLink } from 'lucide-react';
import { formatPrice } from '@/utils';
import { PROVIDER_LABELS, type Provider } from '@/utils/providers';
import { PriceBadge } from './PriceBadge';
import type { HotelPriceComparison } from '@/types';

interface EvidencePanelProps {
  result: HotelPriceComparison;
}

/** Expanded-row evidence table: every room type per source, with
 * source_url + scraped_at + WHO ID (REQ-001 F-011/F-025). */
export function EvidencePanel({ result }: EvidencePanelProps) {
  if (result.prices.length === 0) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
        No price evidence available for this hotel yet.
      </div>
    );
  }

  return (
    <div className="p-4 bg-muted/30">
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left text-muted-foreground">
            <th className="pb-2 pr-4">Source</th>
            <th className="pb-2 pr-4">Room Type</th>
            <th className="pb-2 pr-4">Price (THB)</th>
            <th className="pb-2 pr-4">Meal Plan</th>
            <th className="pb-2 pr-4">Cancellation</th>
            <th className="pb-2 pr-4">Evidence</th>
          </tr>
        </thead>
        <tbody>
          {result.prices.map((entry, idx) => (
            <tr key={`${entry.source}-${idx}`} className="border-t">
              <td className="py-2 pr-4 font-medium">
                {PROVIDER_LABELS[entry.source as Provider] ?? entry.source}
                {entry.who_id && (
                  <div className="text-xs text-muted-foreground">WHO ID: {entry.who_id}</div>
                )}
              </td>
              <td className="py-2 pr-4">
                {entry.room_type}
                <PriceBadge entry={entry} />
              </td>
              <td className="py-2 pr-4">{formatPrice(entry.price_thb)}</td>
              <td className="py-2 pr-4">{entry.meal_plan ?? '-'}</td>
              <td className="py-2 pr-4">{entry.cancellation ?? '-'}</td>
              <td className="py-2 pr-4">
                <div className="flex items-center gap-2">
                  {entry.source_url && (
                    <a
                      href={entry.source_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-1 text-primary hover:underline"
                    >
                      <ExternalLink className="h-3 w-3" /> Source
                    </a>
                  )}
                  <span className="text-xs text-muted-foreground">
                    {new Date(entry.scraped_at).toLocaleString()}
                  </span>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
