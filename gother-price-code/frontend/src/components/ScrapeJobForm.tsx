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
import { cn } from '@/utils';
import type { Device, LoginState, ScrapeMethod } from '@/types';

const LOS_OPTIONS = [1, 2, 3, 5, 7];

const METHOD_CARDS: { value: ScrapeMethod; label: string; description: string; recommended?: boolean }[] = [
  { value: 'serpapi', label: 'SerpAPI', description: 'Google Hotels aggregation (Agoda/Trip)' },
  { value: 'chatgpt', label: 'ChatGPT', description: 'AI knowledge-based rates (bonus method)' },
  { value: 'gemini', label: 'Gemini', description: 'AI knowledge-based rates (bonus method)' },
  { value: 'both', label: 'Both', description: 'SerpAPI + ChatGPT combined', recommended: true },
];

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
  method: ScrapeMethod;
  device: Device;
  login_state: LoginState;
  los_variants: number[];
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
  const [method, setMethod] = useState<ScrapeMethod>('both');
  const [device, setDevice] = useState<Device>('desktop');
  const [loginState, setLoginState] = useState<LoginState>('public');
  const [losVariants, setLosVariants] = useState<number[]>([1]);

  const toggleLos = (nights: number) => {
    setLosVariants((prev) =>
      prev.includes(nights) ? prev.filter((n) => n !== nights) : [...prev, nights].sort((a, b) => a - b)
    );
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({
      checkin_date: checkinDate,
      checkout_date: checkoutDate,
      rooms,
      adults,
      force_refresh: forceRefresh,
      method,
      device,
      login_state: loginState,
      los_variants: losVariants.length > 0 ? losVariants : [1],
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

            {/* Length of Stay Variants */}
            <div className="space-y-2">
              <Label>Length of Stay Variants</Label>
              <div className="flex flex-wrap gap-3">
                {LOS_OPTIONS.map((nights) => (
                  <label
                    key={nights}
                    className="flex items-center gap-1.5 text-sm font-normal cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      checked={losVariants.includes(nights)}
                      onChange={() => toggleLos(nights)}
                      className="h-4 w-4 rounded border-gray-300"
                    />
                    {nights} night{nights !== 1 ? 's' : ''}
                  </label>
                ))}
              </div>
              <p className="text-xs text-muted-foreground">
                Some OTAs offer per-night discounts on longer stays — scrape multiple LOS to catch them.
              </p>
            </div>

            {/* Scraping Method */}
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
                        selected ? 'border-sky-500 bg-sky-50' : 'border-input hover:border-slate-300'
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

            {/* Device & Login State */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="device">Device</Label>
                <select
                  id="device"
                  value={device}
                  onChange={(e) => setDevice(e.target.value as Device)}
                  className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option value="desktop">Desktop</option>
                  <option value="mobile_web">Mobile Web</option>
                </select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="login-state">Login State</Label>
                <select
                  id="login-state"
                  value={loginState}
                  onChange={(e) => setLoginState(e.target.value as LoginState)}
                  className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option value="public">Public</option>
                  <option value="member">Member</option>
                </select>
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
