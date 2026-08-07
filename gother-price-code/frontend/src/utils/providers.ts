// Named providers (REQ-001-v1.2 F-022). Must mirror
// backend/src/scraper/providers.rs::KNOWN_PROVIDERS exactly.
export const KNOWN_PROVIDERS = ['gother', 'agoda', 'trip', 'wink'] as const;
export type Provider = (typeof KNOWN_PROVIDERS)[number];

export const PROVIDER_LABELS: Record<Provider, string> = {
  gother: 'Gother',
  agoda: 'Agoda',
  trip: 'Trip',
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
