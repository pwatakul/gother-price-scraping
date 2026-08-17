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

/** Fire the standard grid immediately (REQ-008 F-010). Additive to the
 * schedule — does not move the next cron run. */
export async function runScheduledScrapeConfig(id: string): Promise<{ jobs_queued: number }> {
  const response = await apiClient.post<{ jobs_queued: number }>(
    `/scheduled-scrape-configs/${id}/run`
  );
  return response.data;
}
