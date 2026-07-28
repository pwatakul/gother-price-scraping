// ===========================================
// Scrape Jobs API
// ===========================================

import apiClient from './client';
import type {
  ScrapeJob,
  ScrapeJobWithProgress,
  ScrapeResultsResponse,
  CreateScrapeJobRequest,
} from '@/types';

// Create scrape job
export async function createScrapeJob(data: CreateScrapeJobRequest): Promise<ScrapeJob> {
  const response = await apiClient.post<ScrapeJob>('/scrape-jobs', data);
  return response.data;
}

// Get scrape job with progress
export async function getScrapeJob(id: string): Promise<ScrapeJobWithProgress> {
  const response = await apiClient.get<ScrapeJobWithProgress>(`/scrape-jobs/${id}`);
  return response.data;
}

// Cancel scrape job
export async function cancelScrapeJob(id: string): Promise<ScrapeJob> {
  const response = await apiClient.delete<ScrapeJob>(`/scrape-jobs/${id}`);
  return response.data;
}

// Get scrape results
export async function getScrapeResults(id: string): Promise<ScrapeResultsResponse> {
  const response = await apiClient.get<ScrapeResultsResponse>(`/scrape-jobs/${id}/results`);
  return response.data;
}

// Export results as Excel
export async function exportResults(id: string): Promise<Blob> {
  const response = await apiClient.get(`/scrape-jobs/${id}/export`, {
    responseType: 'blob',
  });
  return response.data;
}

// Helper to download blob as file
export function downloadBlob(blob: Blob, filename: string): void {
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  window.URL.revokeObjectURL(url);
}
