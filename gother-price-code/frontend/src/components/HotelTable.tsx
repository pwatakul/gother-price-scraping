import { Trash2 } from 'lucide-react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
import { Button } from '@/components/ui/Button';
import { formatPrice, formatRelativeTime } from '@/utils/format';
import type { HotelWithPrice } from '@/types';

interface HotelTableProps {
  hotels: HotelWithPrice[];
  onRemove?: (hotelId: string) => void;
  isRemoving?: boolean;
}

export function HotelTable({ hotels, onRemove, isRemoving }: HotelTableProps) {
  if (hotels.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        No hotels in this group yet. Add hotels manually or import from Excel.
      </div>
    );
  }

  return (
    <div className="border rounded-lg overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50">
            <TableHead>Hotel Name</TableHead>
            <TableHead>City</TableHead>
            <TableHead>Country</TableHead>
            <TableHead className="text-right">Last Price</TableHead>
            <TableHead className="text-right">Source</TableHead>
            <TableHead className="text-right">Last Scraped</TableHead>
            {onRemove && <TableHead className="w-[50px]"></TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {hotels.map((hotel) => (
            <TableRow key={hotel.id}>
              <TableCell className="font-medium">{hotel.name}</TableCell>
              <TableCell>{hotel.city}</TableCell>
              <TableCell>{hotel.country}</TableCell>
              <TableCell className="text-right">
                {hotel.last_price_thb ? (
                  <span className="font-semibold">
                    {formatPrice(hotel.last_price_thb)}
                  </span>
                ) : (
                  <span className="text-muted-foreground">-</span>
                )}
              </TableCell>
              <TableCell className="text-right">
                {hotel.last_price_source || (
                  <span className="text-muted-foreground">-</span>
                )}
              </TableCell>
              <TableCell className="text-right text-muted-foreground">
                {formatRelativeTime(hotel.last_scraped_at)}
              </TableCell>
              {onRemove && (
                <TableCell>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 text-muted-foreground hover:text-destructive"
                    onClick={() => onRemove(hotel.id)}
                    disabled={isRemoving}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
