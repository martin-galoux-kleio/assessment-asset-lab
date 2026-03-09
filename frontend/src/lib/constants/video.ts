/** Max allowed video file size: 1 GB */
export const MAX_VIDEO_SIZE_BYTES = 1024 * 1024 * 1024;

/** MIME type prefix for allowed uploads */
export const VIDEO_MIME_PREFIX = 'video/';

export function isVideoMimeType(type: string): boolean {
  return type.startsWith(VIDEO_MIME_PREFIX);
}

export function isWithinSizeLimit(size: number): boolean {
  return size > 0 && size <= MAX_VIDEO_SIZE_BYTES;
}
