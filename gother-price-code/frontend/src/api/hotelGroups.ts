// ===========================================
// Hotel Groups API
// ===========================================

import apiClient from './client';
import type { SearchConfig } from '@/components/SearchConfigForm';
import type {
  HotelGroup,
  HotelGroupWithCount,
  HotelGroupDetailResponse,
  CreateHotelGroupRequest,
  UpdateHotelGroupRequest,
  Hotel,
  CreateHotelRequest,
  GroupJobsResponse,
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

// Import hotels from Excel (plain hotel_name/city/country format)
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

// Import hotels from the master hotel-list format (HID, Hotel-Name,
// UPDATE URL, SLUG, Supplier-or-Direct, Country, SEARCH — see ADR-003)
export async function importMasterHotels(
  groupId: string,
  file: File
): Promise<{ success: boolean; imported_count: number }> {
  const formData = new FormData();
  formData.append('file', file);

  const response = await apiClient.post(`/hotel-groups/${groupId}/import-master`, formData, {
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

// List scrape jobs for group. Paginated server-side — job history grows
// steadily (a scheduled grid adds 5 rows per fire), so the whole list is
// not fetched just to show a page of it.
export async function listGroupJobs(
  groupId: string,
  limit = 20,
  offset = 0
): Promise<GroupJobsResponse> {
  const response = await apiClient.get<GroupJobsResponse>(`/hotel-groups/${groupId}/jobs`, {
    params: { limit, offset },
  });
  return response.data;
}

// Saved per-group price-search config (ADR-012)
export async function updateSearchConfig(
  groupId: string,
  config: SearchConfig
): Promise<HotelGroup> {
  const response = await apiClient.put<HotelGroup>(
    `/hotel-groups/${groupId}/search-config`,
    config
  );
  return response.data;
}

/** Run the saved search now. Check-in is derived server-side from the
 * stored days-ahead offset, so the caller passes nothing. */
export async function runSavedSearch(
  groupId: string
): Promise<{ jobs_queued: number; job_ids: string[] }> {
  const response = await apiClient.post<{ jobs_queued: number; job_ids: string[] }>(
    `/hotel-groups/${groupId}/search-runs`
  );
  return response.data;
}
