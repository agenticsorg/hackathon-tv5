/**
 * Category visual utilities for recommendation cards
 */

export function getCategoryGradient(category: string): string {
  const gradients: Record<string, string> = {
    meditation: 'from-purple-500 to-indigo-600',
    movie: 'from-red-500 to-pink-600',
    music: 'from-green-500 to-teal-600',
    series: 'from-blue-500 to-cyan-600',
    documentary: 'from-amber-500 to-orange-600',
    short: 'from-rose-500 to-fuchsia-600',
    exercise: 'from-emerald-500 to-lime-600',
    podcast: 'from-violet-500 to-purple-600',
  };
  return gradients[category.toLowerCase()] || 'from-gray-500 to-slate-600';
}

export function getCategoryIcon(category: string): string {
  const icons: Record<string, string> = {
    meditation: '🧘',
    movie: '🎬',
    music: '🎵',
    series: '📺',
    documentary: '📚',
    short: '🎥',
    exercise: '💪',
    podcast: '🎙️',
  };
  return icons[category.toLowerCase()] || '🎭';
}

export function formatDuration(minutes: number): string {
  if (minutes < 60) {
    return `${minutes} min`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
}

export function getScoreColor(score: number): string {
  if (score >= 80) return 'text-green-500';
  if (score >= 60) return 'text-yellow-500';
  return 'text-red-500';
}

export function getScoreBgColor(score: number): string {
  if (score >= 80) return 'bg-green-500/10';
  if (score >= 60) return 'bg-yellow-500/10';
  return 'bg-red-500/10';
}
