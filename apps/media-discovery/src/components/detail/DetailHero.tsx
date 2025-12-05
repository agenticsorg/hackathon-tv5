'use client';

import Image from 'next/image';
import Link from 'next/link';
import { getPosterUrl } from '@/lib/tmdb';
import type { Movie, TVShow, Genre } from '@/types/media';
import type { Video } from '@/lib/tmdb';

interface DetailHeroProps {
  content: Movie | TVShow;
  genres: Genre[];
  videos?: Video[];
}

export function DetailHero({ content, genres, videos }: DetailHeroProps) {
  const posterUrl = getPosterUrl(content.posterPath);
  const trailer = videos?.find(
    (v) => v.type === 'Trailer' && v.site === 'YouTube'
  );

  const year = content.releaseDate
    ? new Date(content.releaseDate).getFullYear()
    : null;

  const runtime =
    content.mediaType === 'movie' && content.runtime
      ? `${Math.floor(content.runtime / 60)}h ${content.runtime % 60}m`
      : null;

  const seasons =
    content.mediaType === 'tv' && content.numberOfSeasons
      ? `${content.numberOfSeasons} Season${content.numberOfSeasons > 1 ? 's' : ''}`
      : null;

  return (
    <div className="relative z-10 -mt-48 md:-mt-64 px-4 md:px-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex flex-col md:flex-row gap-6 md:gap-10">
          {/* Poster */}
          <div className="flex-shrink-0 w-48 md:w-64 mx-auto md:mx-0">
            <div className="relative aspect-[2/3] rounded-xl overflow-hidden shadow-2xl ring-1 ring-white/10">
              {posterUrl ? (
                <Image
                  src={posterUrl}
                  alt={content.title}
                  fill
                  priority
                  className="object-cover"
                  sizes="(max-width: 768px) 192px, 256px"
                />
              ) : (
                <div className="absolute inset-0 bg-gray-800 flex items-center justify-center">
                  <svg
                    className="w-16 h-16 text-gray-600"
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
            </div>
          </div>

          {/* Content */}
          <div className="flex-1 text-center md:text-left">
            {/* Title */}
            <h1 className="text-3xl md:text-4xl lg:text-5xl font-bold text-white mb-2">
              {content.title}
            </h1>

            {/* Tagline */}
            {content.mediaType === 'movie' && content.tagline && (
              <p className="text-lg text-gray-400 italic mb-4">
                {content.tagline}
              </p>
            )}

            {/* Meta info */}
            <div className="flex flex-wrap items-center justify-center md:justify-start gap-3 text-sm text-gray-400 mb-4">
              {year && <span>{year}</span>}
              {(runtime || seasons) && (
                <>
                  <span className="w-1 h-1 rounded-full bg-gray-600" />
                  <span>{runtime || seasons}</span>
                </>
              )}
              {content.voteAverage > 0 && (
                <>
                  <span className="w-1 h-1 rounded-full bg-gray-600" />
                  <span className="flex items-center gap-1">
                    <svg
                      className="w-4 h-4 text-yellow-500"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                    </svg>
                    {content.voteAverage.toFixed(1)}
                  </span>
                </>
              )}
              {content.status && (
                <>
                  <span className="w-1 h-1 rounded-full bg-gray-600" />
                  <span className="px-2 py-0.5 bg-gray-800 rounded text-xs">
                    {content.status}
                  </span>
                </>
              )}
            </div>

            {/* Genres */}
            {genres.length > 0 && (
              <div className="flex flex-wrap justify-center md:justify-start gap-2 mb-6">
                {genres.map((genre) => (
                  <Link
                    key={genre.id}
                    href={`/search?genre=${genre.id}`}
                    className="px-3 py-1 bg-blue-600/20 hover:bg-blue-600/30 text-blue-400 text-sm rounded-full transition-colors"
                  >
                    {genre.name}
                  </Link>
                ))}
              </div>
            )}

            {/* Overview */}
            <p className="text-gray-300 leading-relaxed max-w-3xl mb-6">
              {content.overview || 'No overview available.'}
            </p>

            {/* Actions */}
            <div className="flex flex-wrap justify-center md:justify-start gap-3">
              {trailer && (
                <a
                  href={`https://www.youtube.com/watch?v=${trailer.key}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-2 px-6 py-3 bg-red-600 hover:bg-red-700 text-white font-medium rounded-lg transition-colors"
                >
                  <svg
                    className="w-5 h-5"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fillRule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z"
                      clipRule="evenodd"
                    />
                  </svg>
                  Watch Trailer
                </a>
              )}
              <button className="inline-flex items-center gap-2 px-6 py-3 bg-gray-800 hover:bg-gray-700 text-white font-medium rounded-lg transition-colors">
                <svg
                  className="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 4v16m8-8H4"
                  />
                </svg>
                Add to Watchlist
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
