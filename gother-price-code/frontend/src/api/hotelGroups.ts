// ===========================================
// Hotel Groups API
// ===========================================

import apiClient from './client';
import type {
  HotelGroupWithCount,
  HotelGroupDetailResponse,
  CreateHotelGroupRequest,
  UpdateHotelGroupRequest,
  Hotel,
  CreateHotelRequest,
  ScrapeJob,
} from '@/types';

// List all hotel groups
export async function listHotelGroups(): Promise<HotelGroupWithCount[]> {
  const response = await apiClient.get<HotelGroupWithCount[]>('/hotel-groups');
  return response.data;
}

// Get hotel group by ID with hotels
export async function getHotelGroup(id: string): Promise<HotelGroupDetailResponse> {
  const response = await apiClient.get<HotelGroupDetailResponse>(`/hotel-groups/${id}`);
  return response.data;
}

// Create hotel group
export async function createHotelGroup(data: CreateHotelGroupRequest): Promise<HotelGroupWithCount> {
  const response = await apiClient.post<HotelGroupWithCount>('/hotel-groups', data);
  return response.data;
}

// Create hotel group with Excel file
export async function createHotelGroupWithExcel(
  name: string,
  description: string | undefined,
  file: File
): Promise<HotelGroupWithCount> {
  const formData = new FormData();
  formData.append('name', name);
  if (description) {
    formData.append('description', description);
  }
  formData.append('file', file);

  const response = await apiClient.post<HotelGroupWithCount>('/hotel-groups', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
  return response.data;
}

// Update hotel group
export async function updateHotelGroup(
  id: string,
  data: UpdateHotelGroupRequest
): Promise<HotelGroupWithCount> {
  const response = await apiClient.put<HotelGroupWithCount>(`/hotel-groups/${id}`, data);
  return response.data;
}

// Delete hotel group
export async function deleteHotelGroup(id: string): Promise<void> {
  await apiClient.delete(`/hotel-groups/${id}`);
}

// Import hotels from Excel
export async function importHotels(
  groupId: string,
  file: File
): Promise<{ success: boolean; imported_count: number }> {
  const formData = new FormData();
  formData.append('file', file);

  const response = await apiClient.post(`/hotel-groups/${groupId}/import`, formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
  return response.data;
}

// Add hotel to group
export async function addHotelToGroup(groupId: string, data: CreateHotelRequest): Promise<Hotel> {
  const response = await apiClient.post<Hotel>(`/hotel-groups/${groupId}/hotels`, data);
  return response.data;
}

// Remove hotel from group
export async function removeHotelFromGroup(groupId: string, hotelId: string): Promise<void> {
  await apiClient.delete(`/hotel-groups/${groupId}/hotels/${hotelId}`);
}

// List scrape jobs for group
export async function listGroupJobs(
  groupId: string,
  limit = 20,
  offset = 0
): Promise<ScrapeJob[]> {
  const response = await apiClient.get<ScrapeJob[]>(`/hotel-groups/${groupId}/jobs`, {
    params: { limit, offset },
  });
  return response.data;
}
