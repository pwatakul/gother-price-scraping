import { Link } from 'react-router-dom';
import { Hotel, Calendar, MoreVertical, Trash2 } from 'lucide-react';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { formatRelativeTime } from '@/utils/format';
import type { HotelGroupWithCount } from '@/types';

interface HotelGroupCardProps {
  group: HotelGroupWithCount;
  onDelete?: (id: string) => void;
}

export function HotelGroupCard({ group, onDelete }: HotelGroupCardProps) {
  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <Link to={`/groups/${group.id}`} className="flex-1">
            <CardTitle className="text-lg hover:text-primary transition-colors">
              {group.name}
            </CardTitle>
          </Link>
          {onDelete && (
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 text-muted-foreground hover:text-destructive"
              onClick={(e) => {
                e.preventDefault();
                onDelete(group.id);
              }}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          )}
        </div>
        {group.description && (
          <CardDescription className="line-clamp-2">
            {group.description}
          </CardDescription>
        )}
      </CardHeader>
      <CardContent>
        <div className="flex items-center gap-4 text-sm text-muted-foreground">
          <div className="flex items-center gap-1">
            <Hotel className="h-4 w-4" />
            <span>{group.hotel_count} hotel{group.hotel_count !== 1 ? 's' : ''}</span>
          </div>
          <div className="flex items-center gap-1">
            <Calendar className="h-4 w-4" />
            <span>
              {group.last_scraped_at
                ? `Scraped ${formatRelativeTime(group.last_scraped_at)}`
                : 'Never scraped'}
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
