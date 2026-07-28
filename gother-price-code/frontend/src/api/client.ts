// ===========================================
// API Client - Base Configuration
// ===========================================

import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || '/api';

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Response interceptor for error handling
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response) {
      // Server responded with error
      const message = error.response.data?.error?.message || 'An error occurred';
      console.error('API Error:', message);
    } else if (error.request) {
      // No response received
      console.error('Network Error:', error.message);
    }
    return Promise.reject(error);
  }
);

export default apiClient;
