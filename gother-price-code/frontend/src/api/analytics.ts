// ===========================================
// Analytics API (REQ-003)
// ===========================================

import apiClient from './client';
import type {
  BookingWindowRow,
  HeatmapCell,
  MarketOverview,
  MarketPositionEntry,
  ParityViolationRow,
  PriceHistoryListResponse,
  PriceTrendPoint,
  WinRateRow,
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

export async function getHotelTrend(hotelId: string, days = 90, source?: string): Promise<PriceTrendPoint[]> {
  const response = await apiClient.get<PriceTrendPoint[]>(`/price-history/hotel/${hotelId}/trend`, {
    params: { days, source },
  });
  return response.data;
}

export interface PriceHistoryFilters {
  hotelId: string;
  source?: string;
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
      limit: filters.limit,
      offset: filters.offset,
    },
  });
  return response.data;
}
