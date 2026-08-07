// ===========================================
// Hotel Directory API (REQ-007)
// ===========================================

import apiClient from './client';
import type { HotelDetail, HotelListResponse } from '@/types';

export interface HotelListFilters {
  country?: string;
  city?: string;
  q?: string;
  limit?: number;
  offset?: number;
}

export async function listHotels(filters: HotelListFilters): Promise<HotelListResponse> {
  const response = await apiClient.get<HotelListResponse>('/hotels', { params: filters });
  return response.data;
}

export async function getHotelDetail(id: string): Promise<HotelDetail> {
  const response = await apiClient.get<HotelDetail>(`/hotels/${id}`);
  return response.data;
}

export async function listCountries(): Promise<string[]> {
  const response = await apiClient.get<string[]>('/hotels/countries');
  return response.data;
}

export async function listCities(country?: string): Promise<string[]> {
  const response = await apiClient.get<string[]>('/hotels/cities', { params: { country } });
  return response.data;
}
