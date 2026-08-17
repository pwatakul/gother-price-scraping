// ===========================================
// Hotel Price Scraper - TypeScript Types
// ===========================================

// Hotel Group Types
export interface HotelGroup {
  id: string;
  name: string;
  description: string | null;
  /** Saved price-search config (ADR-012). `search_days_ahead` is an
   * offset from the run date, not a calendar date. */
  search_method: ScrapeMethod;
  search_days_ahead: number[];
  search_rooms: number;
  search_adults: number;
  created_at: string;
  updated_at: string;
}

export interface HotelGroupWithCount {
  id: string;
  name: string;
  description: string | null;
  hotel_count: number;
  last_scraped_at: string | null;
  created_at: string;
}

export interface CreateHotelGroupRequest {
  name: string;
  description?: string;
}

export interface UpdateHotelGroupRequest {
  name?: string;
  description?: string;
}

// Hotel Types
export interface Hotel {
  id: string;
  name: string;
  city: string;
  country: string;
  normalized_name: string;
  hid: number | null;
  slug: string | null;
  update_url: string | null;
  supplier_type: string | null;
  created_at: string;
  updated_at: string;
}

export interface HotelWithPrice {
  id: string;
  name: string;
  city: string;
  country: string;
  last_price_thb: number | null;
  last_price_source: string | null;
  last_scraped_at: string | null;
}

export interface CreateHotelRequest {
  name: string;
  city: string;
  country: string;
}

// Scrape Job Types
export type ScrapeJobStatus = 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';
export type ScrapeMethod = 'serpapi' | 'gemini' | 'both';
export type Device = 'desktop' | 'mobile_web';
export type LoginState = 'public' | 'member';

export interface ScrapeJob {
  id: string;
  hotel_group_id: string;
  checkin_date: string;
  checkout_date: string;
  rooms: number;
  adults: number;
  status: ScrapeJobStatus;
  force_refresh: boolean;
  method: ScrapeMethod;
  los_variants: number[];
  device: Device;
  login_state: LoginState;
  created_at: string;
  completed_at: string | null;
}

/** Paginated scrape-job history for one group. */
export interface GroupJobsResponse {
  jobs: ScrapeJob[];
  total: number;
}

export interface ScrapeProgress {
  total: number;
  completed: number;
  failed: number;
  pending: number;
}

export interface ScrapeJobWithProgress extends Omit<ScrapeJob, 'force_refresh' | 'los_variants'> {
  progress: ScrapeProgress;
}

export interface CreateScrapeJobRequest {
  hotel_group_id: string;
  checkin_date: string;
  checkout_date: string;
  rooms: number;
  adults: number;
  force_refresh?: boolean;
  method?: ScrapeMethod;
  los_variants?: number[];
  device?: Device;
  login_state?: LoginState;
}

// Scrape Results Types
export type HotelScrapeStatus = 'pending' | 'processing' | 'success' | 'failed';

export interface PriceEntry {
  source: string;
  room_type: string;
  price_thb: number;
  original_price: number | null;
  currency: string | null;
  meal_plan: string | null;
  cancellation: string | null;
  source_url: string | null;
  scraped_at: string;
  los_nights: number;
  who_id: string | null;
  is_direct_contract: boolean;
  mismatch_warning: string | null;
}

export interface HotelInfo {
  id: string;
  name: string;
  city: string;
  country: string;
  hid: number | null;
}

export interface HotelPriceComparison {
  hotel: HotelInfo;
  status: HotelScrapeStatus;
  error_message: string | null;
  prices: PriceEntry[];
  best_source: string | null;
  best_price: number | null;
  gother_price: number | null;
  price_difference: number | null;
  price_diff_percent: number | null;
}

export interface ScrapeJobInfo {
  id: string;
  checkin_date: string;
  checkout_date: string;
  rooms: number;
  adults: number;
  status: string;
  method: ScrapeMethod;
  device: Device;
  login_state: LoginState;
  created_at: string;
  completed_at: string | null;
}

export interface ScrapeResultsSummary {
  total_hotels: number;
  successful: number;
  failed: number;
  avg_best_price: number | null;
}

export interface ScrapeResultsResponse {
  job: ScrapeJobInfo;
  summary: ScrapeResultsSummary;
  results: HotelPriceComparison[];
}

// Hotel Directory Types (REQ-007 — global "All Hotels" page)
export interface HotelWithGroupsAndPrice {
  id: string;
  name: string;
  city: string;
  country: string;
  hid: number | null;
  slug: string | null;
  supplier_type: string | null;
  group_names: string[];
  last_price_thb: number | null;
  last_price_source: string | null;
  last_scraped_at: string | null;
}

export interface HotelListResponse {
  hotels: HotelWithGroupsAndPrice[];
  total: number;
}

export interface HotelDetail {
  hotel: Hotel;
  group_names: string[];
  trend: PriceTrendPoint[];
}

// Analytics Types (REQ-003)
export interface MarketOverview {
  total_hotels: number;
  gother_cheapest_pct: number;
  avg_gap_thb: number;
}

export interface MarketPositionEntry {
  hotel_id: string;
  hotel_name: string;
  /** The stay every provider on this row was compared on (ADR-013). */
  checkin_date: string;
  gother_price: number | null;
  best_price: number | null;
  best_source: string | null;
  gap_thb: number | null;
  gap_pct: number | null;
  is_winning: boolean;
  /** Cheapest provider for this stay — populated today, unlike the
   * Gother columns above. */
  cheapest_source: string | null;
  cheapest_price: number | null;
  provider_count: number;
  spread_pct: number | null;
}

export interface HeatmapCell {
  hotel_id: string;
  hotel_name: string;
  source: string;
  checkin_date: string;
  price_thb: number | null;
  gap_pct: number | null;
  /** Cheapest provider for this hotel's stay — the winner highlight. */
  is_cheapest: boolean;
}

export interface WinRateRow {
  hotel_id: string;
  days_won: number;
  days_total: number;
  win_rate_pct: number;
}

export interface BookingWindowRow {
  source: string;
  device: Device;
  days_in_advance: number;
  avg_price_thb: number;
  min_price_thb: number;
  sample_count: number;
}

/** One provider's standing versus the cheapest quote per hotel.
 * `median_premium_pct` is a median, not a mean — a single bad scrape
 * (a five-star resort returned at THB 52) makes an average meaningless. */
export interface ProviderBenchmarkRow {
  source: string;
  /** Stay-level comparisons (hotel + check-in date) this provider took
   * part in, so thin coverage is visible beside a high win rate. */
  quotes_compared: number;
  hotels_covered: number;
  times_cheapest: number;
  cheapest_pct: number;
  median_premium_pct: number;
}

/** A booking window with data, for the trend chart's selector. */
export interface TrendWindow {
  days_in_advance: number;
  sample_count: number;
}

export interface ParityViolationRow {
  hotel_id: string;
  hotel_name: string;
  gother_price: number;
  best_ota_price: number;
  gap_pct: number;
}

export interface PriceTrendPoint {
  source: string;
  day: string;
  /** Days between the scrape and check-in — the booking window this
   * point belongs to (ADR-013). */
  days_in_advance: number;
  avg_price_thb: number;
  min_price_thb: number;
  max_price_thb: number;
  sample_count: number;
}

// Raw price-history row (one per scrape) — REQ-002 F-007
export interface HotelPriceHistoryRow {
  id: string;
  hotel_id: string;
  source: string;
  room_type: string;
  price_thb: number;
  original_price: number | null;
  currency: string | null;
  exchange_rate_id: string;
  meal_plan: string | null;
  cancellation: string | null;
  source_url: string | null;
  checkin_date: string;
  checkout_date: string;
  rooms: number;
  adults: number;
  device: Device;
  /** Which scraper produced this row — 'serpapi' is a real scrape,
   * 'gemini' is an AI estimate used only where scraping found nothing. */
  via_method: string;
  scrape_job_id: string | null;
  scraped_at: string;
}

export interface PriceHistoryListResponse {
  rows: HotelPriceHistoryRow[];
  total: number;
}

// API Response Types
export interface ApiError {
  error: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
}

// Hotel Group Detail Response
export interface HotelGroupDetailResponse {
  group: HotelGroup;
  hotels: HotelWithPrice[];
}

// Scheduled Scrape Config Types (REQ-002 F-003/F-004)
// A schedule configures only *when* and *how* to scrape — the booking
// windows, devices and stay params are fixed system constants (ADR-006).
export interface ScheduledScrapeConfig {
  id: string;
  hotel_group_id: string;
  name: string | null;
  cron_expression: string;
  is_active: boolean;
  last_run_at: string | null;
  next_run_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateScheduledScrapeConfigRequest {
  hotel_group_id: string;
  name?: string;
  cron_expression: string;
  is_active?: boolean;
}
