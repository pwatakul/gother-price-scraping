// ===========================================
// Hotels API
// ===========================================

import apiClient from './client';
import type { Hotel } from '@/types';

// Search hotels
export async function searchHotels(query: string, limit = 10): Promise<Hotel[]> {
  const response = await apiClient.get<Hotel[]>('/hotels/search', {
    params: { q: query, limit },
  });
  return response.data;
}

// Download import template
export async function downloadTemplate(): Promise<Blob> {
  const response = await apiClient.get('/templates/hotel-import', {
    responseType: 'blob',
  });
  return response.data;
}
