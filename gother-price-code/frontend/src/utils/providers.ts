// Allowlisted providers (REQ-001-v1.4 F-022 / ADR-009). Must mirror
// backend/src/scraper/providers.rs::KNOWN_PROVIDERS exactly.
export const KNOWN_PROVIDERS = [
  'gother',
  'direct',
  'agoda',
  'trip',
  'booking',
  'expedia',
  'priceline',
  'traveloka',
  'klook',
  'wink',
] as const;
export type Provider = (typeof KNOWN_PROVIDERS)[number];

export const PROVIDER_LABELS: Record<Provider, string> = {
  gother: 'Gother',
  direct: 'Hotel Direct',
  agoda: 'Agoda',
  trip: 'Trip.com',
  booking: 'Booking.com',
  expedia: 'Expedia',
  priceline: 'Priceline',
  traveloka: 'Traveloka',
  klook: 'Klook',
  wink: 'Wink',
};

// Wink is domestic-only (Thailand) per the brief.
export const DOMESTIC_ONLY_PROVIDERS: Provider[] = ['wink'];

export function isDomestic(country: string): boolean {
  return country.trim().toLowerCase() === 'thailand';
}

export function visibleProviders(country: string): Provider[] {
  const domestic = isDomestic(country);
  return KNOWN_PROVIDERS.filter((p) => domestic || !DOMESTIC_ONLY_PROVIDERS.includes(p));
}
