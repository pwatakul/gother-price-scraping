// ===========================================
// Scheduled Scrape Configs API (REQ-002 F-003/F-004)
// ===========================================

import apiClient from './client';
import type { ScheduledScrapeConfig, CreateScheduledScrapeConfigRequest } from '@/types';

export async function listScheduledScrapeConfigs(
  hotelGroupId: string
): Promise<ScheduledScrapeConfig[]> {
  const response = await apiClient.get<ScheduledScrapeConfig[]>('/scheduled-scrape-configs', {
    params: { hotel_group_id: hotelGroupId },
  });
  return response.data;
}

export async function createScheduledScrapeConfig(
  data: CreateScheduledScrapeConfigRequest
): Promise<ScheduledScrapeConfig> {
  const response = await apiClient.post<ScheduledScrapeConfig>('/scheduled-scrape-configs', data);
  return response.data;
}

export async function updateScheduledScrapeConfig(
  id: string,
  data: Partial<CreateScheduledScrapeConfigRequest>
): Promise<ScheduledScrapeConfig> {
  const response = await apiClient.put<ScheduledScrapeConfig>(
    `/scheduled-scrape-configs/${id}`,
    data
  );
  return response.data;
}

export async function deleteScheduledScrapeConfig(id: string): Promise<void> {
  await apiClient.delete(`/scheduled-scrape-configs/${id}`);
}
