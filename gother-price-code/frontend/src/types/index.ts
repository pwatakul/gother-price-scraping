// ===========================================
// Hotel Price Scraper - TypeScript Types
// ===========================================

// Hotel Group Types
export interface HotelGroup {
  id: string;
  name: string;
  description: string | null;
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

export interface ScrapeJob {
  id: string;
  hotel_group_id: string;
  checkin_date: string;
  checkout_date: string;
  rooms: number;
  adults: number;
  status: ScrapeJobStatus;
  force_refresh: boolean;
  created_at: string;
  completed_at: string | null;
}

export interface ScrapeProgress {
  total: number;
  completed: number;
  failed: number;
  pending: number;
}

export interface ScrapeJobWithProgress extends ScrapeJob {
  progress: ScrapeProgress;
}

export interface CreateScrapeJobRequest {
  hotel_group_id: string;
  checkin_date: string;
  checkout_date: string;
  rooms: number;
  adults: number;
  force_refresh?: boolean;
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
}

export interface HotelInfo {
  id: string;
  name: string;
  city: string;
  country: string;
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
}

export interface ScrapeJobInfo {
  id: string;
  checkin_date: string;
  checkout_date: string;
  rooms: number;
  adults: number;
  status: string;
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
