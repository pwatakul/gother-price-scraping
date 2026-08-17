// ===========================================
// Hotel Directory API (REQ-007)
// ===========================================

import apiClient from './client';
import type { Hotel, HotelDetail, HotelListResponse } from '@/types';

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

/** Edit a hotel's name, city or country. The backend recomputes the
 * normalized name used for matching, and rejects an edit that would
 * collide with another hotel in the same city. */
export async function updateHotel(
  id: string,
  data: { name?: string; city?: string; country?: string }
): Promise<Hotel> {
  const response = await apiClient.put<Hotel>(`/hotels/${id}`, data);
  return response.data;
}
