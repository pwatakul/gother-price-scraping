import { useState } from 'react';
import { format, addDays } from 'date-fns';
import { Calendar, Users, BedDouble } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/Dialog';

interface ScrapeJobFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (data: ScrapeJobFormData) => void;
  isLoading?: boolean;
}

export interface ScrapeJobFormData {
  checkin_date: string;
  checkout_date: string;
  rooms: number;
  adults: number;
  force_refresh: boolean;
}

export function ScrapeJobForm({
  open,
  onOpenChange,
  onSubmit,
  isLoading,
}: ScrapeJobFormProps) {
  const tomorrow = addDays(new Date(), 1);
  const dayAfter = addDays(new Date(), 2);

  const [checkinDate, setCheckinDate] = useState(format(tomorrow, 'yyyy-MM-dd'));
  const [checkoutDate, setCheckoutDate] = useState(format(dayAfter, 'yyyy-MM-dd'));
  const [rooms, setRooms] = useState(1);
  const [adults, setAdults] = useState(2);
  const [forceRefresh, setForceRefresh] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({
      checkin_date: checkinDate,
      checkout_date: checkoutDate,
      rooms,
      adults,
      force_refresh: forceRefresh,
    });
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>Run Price Scrape</DialogTitle>
            <DialogDescription>
              Configure search parameters for price comparison
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {/* Check-in Date */}
            <div className="space-y-2">
              <Label htmlFor="checkin" className="flex items-center gap-2">
                <Calendar className="h-4 w-4" />
                Check-in Date
              </Label>
              <Input
                id="checkin"
                type="date"
                value={checkinDate}
                onChange={(e) => setCheckinDate(e.target.value)}
                min={format(tomorrow, 'yyyy-MM-dd')}
                required
              />
            </div>

            {/* Check-out Date */}
            <div className="space-y-2">
              <Label htmlFor="checkout" className="flex items-center gap-2">
                <Calendar className="h-4 w-4" />
                Check-out Date
              </Label>
              <Input
                id="checkout"
                type="date"
                value={checkoutDate}
                onChange={(e) => setCheckoutDate(e.target.value)}
                min={checkinDate}
                required
              />
            </div>

            {/* Rooms & Adults */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="rooms" className="flex items-center gap-2">
                  <BedDouble className="h-4 w-4" />
                  Rooms
                </Label>
                <Input
                  id="rooms"
                  type="number"
                  value={rooms}
                  onChange={(e) => setRooms(parseInt(e.target.value) || 1)}
                  min={1}
                  max={10}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="adults" className="flex items-center gap-2">
                  <Users className="h-4 w-4" />
                  Adults
                </Label>
                <Input
                  id="adults"
                  type="number"
                  value={adults}
                  onChange={(e) => setAdults(parseInt(e.target.value) || 1)}
                  min={1}
                  max={10}
                  required
                />
              </div>
            </div>

            {/* Force Refresh */}
            <div className="flex items-center gap-2">
              <input
                id="force-refresh"
                type="checkbox"
                checked={forceRefresh}
                onChange={(e) => setForceRefresh(e.target.checked)}
                className="h-4 w-4 rounded border-gray-300"
              />
              <Label htmlFor="force-refresh" className="font-normal">
                Force refresh (ignore cached prices)
              </Label>
            </div>
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? 'Starting...' : 'Start Scraping'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
