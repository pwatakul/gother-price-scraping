// ===========================================
// Analytics API (REQ-003)
// ===========================================

import apiClient from './client';
import type {
  BookingWindowRow,
  Device,
  HeatmapCell,
  MarketOverview,
  MarketPositionEntry,
  ParityViolationRow,
  PriceHistoryListResponse,
  PriceTrendPoint,
  WinRateRow,
  ProviderBenchmarkRow,
  TrendWindow,
} from '@/types';

export async function getOverview(hotelGroupId?: string): Promise<MarketOverview> {
  const response = await apiClient.get<MarketOverview>('/analytics/overview', {
    params: { hotel_group_id: hotelGroupId },
  });
  return response.data;
}

export async function getMarketPosition(hotelGroupId?: string): Promise<MarketPositionEntry[]> {
  const response = await apiClient.get<MarketPositionEntry[]>('/analytics/market-position', {
    params: { hotel_group_id: hotelGroupId },
  });
  return response.data;
}

export async function getHeatmap(hotelGroupId?: string): Promise<HeatmapCell[]> {
  const response = await apiClient.get<HeatmapCell[]>('/analytics/heatmap', {
    params: { hotel_group_id: hotelGroupId },
  });
  return response.data;
}

export async function getWinRate(): Promise<WinRateRow[]> {
  const response = await apiClient.get<WinRateRow[]>('/analytics/win-rate');
  return response.data;
}

/** Gother-independent leaderboard: how often each provider is cheapest,
 * and its median premium over the cheapest. Works before Gother has a
 * price source, unlike win-rate and parity which are defined against it. */
export async function getProviderBenchmark(hotelGroupId?: string): Promise<ProviderBenchmarkRow[]> {
  const response = await apiClient.get<ProviderBenchmarkRow[]>('/analytics/provider-benchmark', {
    params: { hotel_group_id: hotelGroupId },
  });
  return response.data;
}

export async function getParityViolations(threshold = 5.0): Promise<ParityViolationRow[]> {
  const response = await apiClient.get<ParityViolationRow[]>('/analytics/parity-violations', {
    params: { threshold },
  });
  return response.data;
}

export async function getBookingWindow(hotelId: string): Promise<BookingWindowRow[]> {
  const response = await apiClient.get<BookingWindowRow[]>(`/analytics/booking-window/${hotelId}`);
  return response.data;
}

/** `bookingWindow` keeps every series on the same days-in-advance, so
 * providers are compared like for like (ADR-013). */
export async function getHotelTrend(
  hotelId: string,
  days = 90,
  source?: string,
  bookingWindow?: number
): Promise<PriceTrendPoint[]> {
  const response = await apiClient.get<PriceTrendPoint[]>(`/price-history/hotel/${hotelId}/trend`, {
    params: { days, source, booking_window: bookingWindow },
  });
  return response.data;
}

/** Booking windows that actually have data for this hotel, most samples
 * first — the selector is built from these, not a hardcoded list. */
export async function getTrendWindows(hotelId: string): Promise<TrendWindow[]> {
  const response = await apiClient.get<TrendWindow[]>(
    `/price-history/hotel/${hotelId}/trend/windows`
  );
  return response.data;
}

export interface PriceHistoryFilters {
  hotelId: string;
  source?: string;
  device?: Device;
  limit: number;
  offset: number;
}

/** Full, unaggregated price history for one hotel (every individual
 * scrape) — the "all price data" table on the hotel detail page. */
export async function listPriceHistory(filters: PriceHistoryFilters): Promise<PriceHistoryListResponse> {
  const response = await apiClient.get<PriceHistoryListResponse>('/price-history', {
    params: {
      hotel_id: filters.hotelId,
      source: filters.source,
      device: filters.device,
      limit: filters.limit,
      offset: filters.offset,
    },
  });
  return response.data;
}
