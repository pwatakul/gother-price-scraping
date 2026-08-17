import { useEffect, useState } from 'react';
import { CalendarClock, Users, BedDouble } from 'lucide-react';
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
import { cn } from '@/utils';
import type { ScrapeMethod } from '@/types';

/** The scheduler's standard grid. Manual runs pick from the same set so
 * their data lands on the same windows, keeping the booking-window chart
 * and provider benchmark comparable (ADR-013). */
const WINDOW_OPTIONS = [1, 3, 7, 14, 30];

const METHOD_CARDS: { value: ScrapeMethod; label: string; description: string; recommended?: boolean }[] = [
  {
    value: 'serpapi',
    label: 'SerpAPI',
    description: 'Live Google Hotels rates',
    recommended: true,
  },
  { value: 'gemini', label: 'Gemini', description: 'AI knowledge-based; declines if unsure' },
  { value: 'both', label: 'Both', description: 'SerpAPI first, Gemini only fills blanks' },
];

/** The group's saved search settings (ADR-012). Dates are a days-ahead
 * offset rather than calendar dates, so a saved search never goes stale. */
export interface SearchConfig {
  search_method: ScrapeMethod;
  /** Booking windows to scrape, one job (and one night) each. */
  search_days_ahead: number[];
  search_rooms: number;
  search_adults: number;
}

interface SearchConfigFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Current saved values, used to seed the form each time it opens. */
  config: SearchConfig;
  onSubmit: (config: SearchConfig) => void;
  isLoading?: boolean;
  /** Hotels in the group — used to show what a run will cost in searches. */
  hotelCount: number;
}

export function SearchConfigForm({
  open,
  onOpenChange,
  config,
  onSubmit,
  isLoading,
  hotelCount,
}: SearchConfigFormProps) {
  const [windows, setWindows] = useState<number[]>(config.search_days_ahead);
  const [rooms, setRooms] = useState(config.search_rooms);
  const [adults, setAdults] = useState(config.search_adults);
  const [method, setMethod] = useState<ScrapeMethod>(config.search_method);

  // Re-seed whenever the dialog opens, so reopening after a save (or a
  // cancelled edit) always shows what is actually stored.
  useEffect(() => {
    if (!open) return;
    setWindows(config.search_days_ahead);
    setRooms(config.search_rooms);
    setAdults(config.search_adults);
    setMethod(config.search_method);
  }, [open, config]);

  const toggleWindow = (days: number) => {
    setWindows((prev) =>
      prev.includes(days) ? prev.filter((d) => d !== days) : [...prev, days].sort((a, b) => a - b)
    );
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({
      search_method: method,
      search_days_ahead: windows,
      search_rooms: rooms,
      search_adults: adults,
    });
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>Price Search Settings</DialogTitle>
            <DialogDescription>
              Saved for this group. "Run Price Search" uses these, and scheduled runs use the method.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {/* Offsets, not fixed dates, so a saved search never goes
                stale — and drawn from the scheduler's standard set so
                manual and scheduled runs share the same windows. */}
            <div className="space-y-2">
              <Label className="flex items-center gap-2">
                <CalendarClock className="h-4 w-4" />
                Booking windows
              </Label>
              <div className="flex flex-wrap gap-3">
                {WINDOW_OPTIONS.map((days) => (
                  <label
                    key={days}
                    className="flex items-center gap-1.5 text-sm font-normal cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      checked={windows.includes(days)}
                      onChange={() => toggleWindow(days)}
                      className="h-4 w-4 rounded border-gray-300"
                    />
                    +{days} day{days !== 1 ? 's' : ''}
                  </label>
                ))}
              </div>
              <p className="text-xs text-muted-foreground">
                Check-in is this many days from the day you run the search. One search per window,
                one night each — so a run covers several points on the booking curve.
              </p>
              {windows.length === 0 ? (
                <p className="text-xs text-red-600">Select at least one window.</p>
              ) : (
                <p className="text-xs text-muted-foreground">
                  {windows.length} window{windows.length !== 1 ? 's' : ''} × {hotelCount} hotel
                  {hotelCount !== 1 ? 's' : ''} ={' '}
                  <span className="font-semibold text-foreground">
                    {windows.length * hotelCount} searches
                  </span>{' '}
                  per run.
                </p>
              )}
            </div>

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
            <p className="text-xs text-muted-foreground -mt-2">
              Stay length is fixed at 1 night, so every window measures the same product.
            </p>

            <div className="space-y-2">
              <Label>Scraping Method</Label>
              <div className="grid grid-cols-2 gap-3">
                {METHOD_CARDS.map((card) => {
                  const selected = method === card.value;
                  return (
                    <label
                      key={card.value}
                      className={cn(
                        'relative border rounded-[8px] p-3 cursor-pointer transition-colors',
                        selected ? 'border-brand-600 bg-brand-50' : 'border-input hover:border-slate-300'
                      )}
                    >
                      <input
                        type="radio"
                        name="method"
                        value={card.value}
                        checked={selected}
                        onChange={() => setMethod(card.value)}
                        className="sr-only"
                      />
                      {card.recommended && (
                        <span className="absolute -top-2 right-2 rounded-full bg-yellow-100 px-2 py-0.5 text-[10px] font-semibold text-[#854d0e]">
                          ⭐ Recommended
                        </span>
                      )}
                      <div className="text-sm font-semibold">{card.label}</div>
                      <div className="text-xs text-muted-foreground mt-0.5">{card.description}</div>
                    </label>
                  );
                })}
              </div>
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
            <Button type="submit" disabled={isLoading || windows.length === 0}>
              {isLoading ? 'Saving...' : 'Save Settings'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

/** One-line summary of what a run will do, e.g.
 * "SerpAPI · +1/+7/+30 days ahead · 1 night · 1 room, 2 adults". */
export function describeSearchConfig(c: SearchConfig): string {
  const methodLabel =
    METHOD_CARDS.find((m) => m.value === c.search_method)?.label ?? c.search_method;
  const windows = c.search_days_ahead.length
    ? c.search_days_ahead.map((d) => `+${d}`).join('/') + ' days ahead'
    : 'no windows selected';
  return `${methodLabel} · ${windows} · 1 night · ${c.search_rooms} room${c.search_rooms !== 1 ? 's' : ''}, ${c.search_adults} adult${c.search_adults !== 1 ? 's' : ''}`;
}
