import { AlertTriangle, Handshake } from 'lucide-react';
import type { PriceEntry } from '@/types';

interface PriceBadgeProps {
  entry: PriceEntry;
}

/** ⚠️ mismatch badge + direct-contract badge (REQ-001 F-011/F-026). */
export function PriceBadge({ entry }: PriceBadgeProps) {
  if (!entry.mismatch_warning && !entry.is_direct_contract) {
    return null;
  }

  return (
    <span className="inline-flex items-center gap-1 ml-1">
      {entry.mismatch_warning && (
        <span title={entry.mismatch_warning}>
          <AlertTriangle className="h-3.5 w-3.5 text-amber-600" />
        </span>
      )}
      {entry.is_direct_contract && (
        <span title="Direct-contract rate">
          <Handshake className="h-3.5 w-3.5 text-blue-600" />
        </span>
      )}
    </span>
  );
}
