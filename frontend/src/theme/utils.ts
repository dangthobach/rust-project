/**
 * Utility for merging Tailwind classes
 */
export function cn(...inputs: (string | undefined | null | false)[]) {
  return inputs.filter(Boolean).join(' ');
}

/**
 * Get status color class for tasks
 */
export function getTaskStatusClass(status: string): string {
  const statusMap: Record<string, string> = {
    todo: 'bg-neutral-concrete text-black',
    in_progress: 'bg-secondary text-white',
    done: 'bg-primary text-black',
    cancelled: 'bg-neutral-gray text-white',
  };
  return statusMap[status] || statusMap.todo;
}

/**
 * Get priority color class
 */
export function getPriorityClass(priority: string): string {
  const priorityMap: Record<string, string> = {
    low: 'bg-neutral-concrete text-black',
    medium: 'bg-accent-yellow text-black',
    high: 'bg-accent-orange text-white',
    urgent: 'bg-red-500 text-white',
  };
  return priorityMap[priority] || priorityMap.medium;
}

/**
 * Get client status color class
 */
export function getClientStatusClass(status: string): string {
  const statusMap: Record<string, string> = {
    active: 'bg-primary text-black',
    inactive: 'bg-neutral-gray text-white',
    prospect: 'bg-accent-yellow text-black',
    customer: 'bg-secondary text-white',
  };
  return statusMap[status] || statusMap.active;
}

/**
 * Format date to localized string
 */
export function formatDate(date: string | Date): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return d.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * Format date with time
 */
export function formatDateTime(date: string | Date): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return d.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/**
 * Get initials from name
 */
export function getInitials(name: string): string {
  return name
    .split(' ')
    .map((n) => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);
}

/**
 * Truncate text with ellipsis
 */
export function truncate(text: string, length: number): string {
  if (text.length <= length) return text;
  return text.slice(0, length) + '...';
}

/**
 * Format file size
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Debounce function
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout;
  return (...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

/**
 * Generate random brutal shadow offset
 */
export function getRandomBrutalShadow(): string {
  const offsets = ['4px', '8px', '12px', '16px'];
  const x = offsets[Math.floor(Math.random() * offsets.length)];
  const y = offsets[Math.floor(Math.random() * offsets.length)];
  return `${x} ${y} 0px 0px #000000`;
}
