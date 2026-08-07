import { Link, useNavigate } from 'react-router-dom';
import { Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { formatRelativeTime } from '@/utils/format';
import type { HotelGroupWithCount } from '@/types';

interface HotelGroupCardProps {
  group: HotelGroupWithCount;
  onDelete?: (id: string) => void;
}

export function HotelGroupCard({ group, onDelete }: HotelGroupCardProps) {
  const navigate = useNavigate();

  return (
    <div
      className="bg-white border border-slate-200 rounded-[10px] p-[18px_20px] flex items-center gap-3.5 cursor-pointer transition-shadow hover:shadow-[0_4px_12px_rgba(0,0,0,0.08)] hover:border-sky-300"
      onClick={() => navigate(`/groups/${group.id}`)}
    >
      <div className="h-[42px] w-[42px] shrink-0 rounded-[10px] bg-sky-50 flex items-center justify-center text-xl">
        🏨
      </div>

      <div className="flex-1 min-w-0">
        <Link
          to={`/groups/${group.id}`}
          onClick={(e) => e.stopPropagation()}
          className="text-sm font-semibold text-slate-900 hover:text-sky-600 transition-colors"
        >
          {group.name}
        </Link>
        <div className="text-xs text-slate-500 mt-0.5 truncate">
          {group.hotel_count} hotel{group.hotel_count !== 1 ? 's' : ''} ·{' '}
          {group.last_scraped_at
            ? `Last scraped ${formatRelativeTime(group.last_scraped_at)}`
            : 'Never scraped'}
        </div>
        {group.description && (
          <div className="text-xs text-slate-400 mt-1 truncate">{group.description}</div>
        )}
      </div>

      <div className="flex flex-col gap-1.5 shrink-0">
        <Button
          variant="secondary"
          size="sm"
          onClick={(e) => {
            e.stopPropagation();
            navigate(`/groups/${group.id}`);
          }}
        >
          View
        </Button>
        {onDelete && (
          <Button
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-destructive"
            onClick={(e) => {
              e.stopPropagation();
              onDelete(group.id);
            }}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        )}
      </div>
    </div>
  );
}
