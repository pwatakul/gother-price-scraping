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
  // The session cookie is httpOnly, so it has to ride along automatically —
  // there is no token for JavaScript to attach by hand.
  withCredentials: true,
});

// Response interceptor for error handling
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response) {
      // Server responded with error
      const message = error.response.data?.error?.message || 'An error occurred';
      console.error('API Error:', message);

      // Session expired or missing: send the user to the login screen rather
      // than leaving a page of failed requests on screen. The auth endpoints
      // are excluded — a wrong password on /auth/login must surface as an
      // error in the form, and the initial /auth/me probe returns 401 as its
      // normal "not signed in" answer.
      const url: string = error.config?.url ?? '';
      const isAuthCall = url.startsWith('/auth/');
      if (error.response.status === 401 && !isAuthCall && window.location.pathname !== '/login') {
        window.location.href = '/login';
      }
    } else if (error.request) {
      // No response received
      console.error('Network Error:', error.message);
    }
    return Promise.reject(error);
  }
);

export default apiClient;
