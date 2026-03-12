import { writable } from 'svelte/store';

const WATCH_PATTERN = /^\/watch\/([^/]+)\/?$/;
const UPLOAD_PATTERN = /^\/upload\/?$/;

function getRoute(): { name: 'upload' } | { name: 'watch'; params: { id: string } } {
  if (typeof window === 'undefined') return { name: 'upload' };
  const path = window.location.pathname;

  if (path === '/' || path === '') {
    window.history.replaceState({}, '', '/upload');
    return { name: 'upload' };
  }
  if (UPLOAD_PATTERN.test(path)) return { name: 'upload' };
  const match = path.match(WATCH_PATTERN);
  if (match) return { name: 'watch', params: { id: decodeURIComponent(match[1]) } };
  return { name: 'upload' };
}

export const route = writable(getRoute());

export function navigate(path: string): void {
  if (typeof window === 'undefined') return;
  window.history.pushState({}, '', path);
  route.set(getRoute());
}

function onPopState(): void {
  route.set(getRoute());
}

if (typeof window !== 'undefined') {
  window.addEventListener('popstate', onPopState);
  // Sync route on initial load (e.g. direct navigation to /watch/123)
  route.set(getRoute());
}
