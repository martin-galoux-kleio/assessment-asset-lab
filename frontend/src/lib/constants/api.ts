/**
 * Backend API base URL. Empty in dev (Vite proxy handles /api).
 * Set VITE_API_BASE_URL in production (e.g. on Vercel) to your backend origin.
 */
export const API_BASE = (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? '';

export function apiPath(path: string): string {
  const normalized = path.startsWith('/') ? path : `/${path}`;
  return `${API_BASE}${normalized}`;
}
