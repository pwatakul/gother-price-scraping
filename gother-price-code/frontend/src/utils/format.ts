// ===========================================
// Format Utilities
// ===========================================

import { format, parseISO } from 'date-fns';

// Format price in THB
export function formatPrice(price: number | null | undefined): string {
  if (price === null || price === undefined) {
    return 'N/A';
  }
  return `฿${price.toLocaleString('en-US', {
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  })}`;
}

// Format date
export function formatDate(dateString: string | null | undefined): string {
  if (!dateString) {
    return 'N/A';
  }
  try {
    return format(parseISO(dateString), 'MMM d, yyyy');
  } catch {
    return dateString;
  }
}

// Format date time
export function formatDateTime(dateString: string | null | undefined): string {
  if (!dateString) {
    return 'N/A';
  }
  try {
    return format(parseISO(dateString), 'MMM d, yyyy h:mm a');
  } catch {
    return dateString;
  }
}

// Format relative time
export function formatRelativeTime(dateString: string | null | undefined): string {
  if (!dateString) {
    return 'Never';
  }
  
  try {
    const date = parseISO(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMins < 1) {
      return 'Just now';
    }
    if (diffMins < 60) {
      return `${diffMins} min${diffMins === 1 ? '' : 's'} ago`;
    }
    if (diffHours < 24) {
      return `${diffHours} hour${diffHours === 1 ? '' : 's'} ago`;
    }
    if (diffDays < 7) {
      return `${diffDays} day${diffDays === 1 ? '' : 's'} ago`;
    }
    return formatDate(dateString);
  } catch {
    return dateString;
  }
}

// Format percentage
export function formatPercentage(value: number): string {
  return `${Math.round(value)}%`;
}

// Get status color
export function getStatusColor(status: string): string {
  switch (status.toLowerCase()) {
    case 'success':
    case 'completed':
      return 'text-green-600 bg-green-100';
    case 'failed':
    case 'cancelled':
      return 'text-red-600 bg-red-100';
    case 'processing':
      return 'text-blue-600 bg-blue-100';
    case 'pending':
      return 'text-yellow-600 bg-yellow-100';
    default:
      return 'text-gray-600 bg-gray-100';
  }
}

// Capitalize first letter
export function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}
