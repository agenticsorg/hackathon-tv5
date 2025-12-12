'use client';

import Link from 'next/link';
import Image from 'next/image';
import type { MediaContent } from '@/types/media';
import { getPosterUrl } from '@/lib/tmdb';

interface MediaCardProps {
  content: MediaContent;
  reason?: string;
}

export function MediaCard({ content, reason }: MediaCardProps) {
  const posterUrl = getPosterUrl(content.posterPath);
  const href =
    content.mediaType === 'movie'
      ? `/movie/${content.id}`
      : `/tv/${content.id}`;

  return (
    <Link href={href} className="group block">
      <div className="relative aspect-[2/3] rounded-lg overflow-hidden bg-gray-200 dark:bg-gray-800">
        {posterUrl ? (
          <Image
            src={posterUrl}
            alt={content.title}
            fill
            className="object-cover transition-transform group-hover:scale-105"
            sizes="(max-width: 768px) 50vw, (max-width: 1200px) 25vw, 16vw"
          />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center text-gray-400">
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
          <div className="absolute top-2 right-2 px-1.5 py-0.5 bg-black/70 text-white text-xs font-medium rounded">
            {content.voteAverage.toFixed(1)}
          </div>
        )}

        {/* Media type badge */}
        <div className="absolute top-2 left-2 px-1.5 py-0.5 bg-blue-600/90 text-white text-xs font-medium rounded uppercase">
          {content.mediaType}
        </div>

        {/* Hover overlay */}
        <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
          <div className="absolute bottom-0 left-0 right-0 p-3">
            <h3 className="text-white font-medium text-sm line-clamp-2">
              {content.title}
            </h3>
            {content.releaseDate && (
              <p className="text-gray-300 text-xs mt-1">
                {new Date(content.releaseDate).getFullYear()}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Title and reason below card */}
      <div className="mt-2">
        <h3 className="font-medium text-sm line-clamp-1">{content.title}</h3>
        {reason && (
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-1">
            {reason}
          </p>
        )}
      </div>
    </Link>
  );
}
