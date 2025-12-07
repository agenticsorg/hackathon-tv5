'use client';

import Link from 'next/link';
import Image from 'next/image';
import type { MediaContent } from '@/types/media';
import { getPosterUrl } from '@/lib/tmdb';

interface MediaCardProps {
  content: MediaContent;
  reason?: string;
  index?: number;
}

// Map genre IDs to colors
const genreColorMap: Record<number, string> = {
  10749: '#FF6B6B', // Romance
  53: '#4ECDC4',    // Thriller
  35: '#FFE66D',    // Comedy
  878: '#A855F7',   // Sci-Fi
  18: '#F97316',    // Drama
  28: '#EF4444',    // Action
  27: '#6B7280',    // Horror
  99: '#10B981',    // Documentary
};

export function MediaCard({ content, reason, index = 0 }: MediaCardProps) {
  const posterUrl = getPosterUrl(content.posterPath);
  const href =
    content.mediaType === 'movie'
      ? `/movie/${content.id}`
      : `/tv/${content.id}`;

  // Get genre color from first genre
  const primaryGenreId = content.genreIds?.[0];
  const genreColor = primaryGenreId ? genreColorMap[primaryGenreId] : '#4ECDC4';

  return (
    <Link
      href={href}
      className="group block animate-card-reveal"
      style={{ animationDelay: `${index * 80}ms` }}
    >
      <div className="card-hover relative aspect-[2/3] rounded-lg overflow-hidden bg-bg-elevated border-2 border-transparent hover:border-accent-cyan/50">
        {posterUrl ? (
          <Image
            src={posterUrl}
            alt={content.title}
            fill
            className="object-cover"
            sizes="(max-width: 768px) 50vw, (max-width: 1200px) 25vw, 16vw"
          />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center text-text-secondary">
            <svg
              className="w-12 h-12"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"
              />
            </svg>
          </div>
        )}

        {/* Rating badge */}
        {content.voteAverage > 0 && (
          <div className="absolute top-2 right-2 px-2 py-1 bg-bg-deep/80 backdrop-blur-sm text-accent-cyan text-xs font-bold rounded flex items-center gap-1">
            <svg className="w-3 h-3 fill-genre-comedy" viewBox="0 0 20 20">
              <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
            </svg>
            {content.voteAverage.toFixed(1)}
          </div>
        )}

        {/* Media type badge */}
        <div
          className="absolute top-2 left-2 px-2 py-1 text-bg-deep text-xs font-bold rounded uppercase"
          style={{ backgroundColor: genreColor }}
        >
          {content.mediaType}
        </div>

        {/* Hover overlay with genre label */}
        <div className="absolute inset-0 bg-gradient-to-t from-bg-deep via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300">
          <div className="absolute bottom-0 left-0 right-0 p-4">
            <h3 className="text-text-primary font-semibold text-sm line-clamp-2 mb-1">
              {content.title}
            </h3>
            {content.releaseDate && (
              <p className="text-text-secondary text-xs">
                {new Date(content.releaseDate).getFullYear()}
              </p>
            )}
          </div>
        </div>

        {/* Genre color accent bar */}
        <div
          className="absolute bottom-0 left-0 right-0 h-1 opacity-0 group-hover:opacity-100 transition-opacity"
          style={{ backgroundColor: genreColor }}
        />
      </div>

      {/* Title and reason below card */}
      <div className="mt-3">
        <h3 className="font-medium text-sm text-text-primary line-clamp-1">
          {content.title}
        </h3>
        {reason && (
          <p className="text-xs text-text-secondary mt-1 line-clamp-1">
            {reason}
          </p>
        )}
      </div>
    </Link>
  );
}
