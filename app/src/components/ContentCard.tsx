'use client';

import { useState } from 'react';
import Image from 'next/image';
import Link from 'next/link';
import type { Content } from '@/lib/db';

interface ContentCardProps {
  content: Content;
  onLike?: (id: string) => void;
  onSkip?: (id: string) => void;
  onViewSimilar?: (id: string) => void;
}

export function ContentCard({ content, onLike, onSkip, onViewSimilar }: ContentCardProps) {
  const [actionState, setActionState] = useState<'idle' | 'liked' | 'skipped'>('idle');
  const [isAnimating, setIsAnimating] = useState(false);

  const getTypeStyle = (type: string) => {
    if (type === 'series') {
      return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30';
    }
    return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
  };

  const handleWatch = () => {
    setIsAnimating(true);
    setActionState('liked');
    onLike?.(content.id);

    // Reset after animation
    setTimeout(() => {
      setIsAnimating(false);
    }, 2000);
  };

  const handleSkip = () => {
    setIsAnimating(true);
    setActionState('skipped');
    onSkip?.(content.id);

    setTimeout(() => {
      setIsAnimating(false);
    }, 2000);
  };

  return (
    <div className={`
      content-card glass-card rounded-2xl overflow-hidden group relative
      transition-all duration-500
      ${actionState === 'liked' ? 'ring-2 ring-emerald-500/50' : ''}
      ${actionState === 'skipped' ? 'opacity-50' : ''}
    `}>
      {/* Success overlay */}
      {actionState === 'liked' && isAnimating && (
        <div className="absolute inset-0 z-20 flex items-center justify-center bg-emerald-500/20 backdrop-blur-sm animate-pulse">
          <div className="flex flex-col items-center gap-2">
            <div className="w-16 h-16 rounded-full bg-emerald-500/30 flex items-center justify-center animate-bounce">
              <svg className="w-8 h-8 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <span className="text-emerald-400 font-medium">Added to watchlist!</span>
            <span className="text-xs text-zinc-400">AI is learning your preferences</span>
          </div>
        </div>
      )}

      {/* Skip overlay */}
      {actionState === 'skipped' && isAnimating && (
        <div className="absolute inset-0 z-20 flex items-center justify-center bg-zinc-900/50 backdrop-blur-sm">
          <div className="flex flex-col items-center gap-2">
            <div className="w-12 h-12 rounded-full bg-zinc-700/50 flex items-center justify-center">
              <svg className="w-6 h-6 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </div>
            <span className="text-zinc-400 text-sm">Skipped</span>
          </div>
        </div>
      )}

      {/* Clickable area - links to detail page */}
      <Link href={`/content/${content.id}`} className="block cursor-pointer">
        {/* Header with poster image */}
        <div className="relative h-48 bg-gradient-to-br from-zinc-800 to-zinc-900 overflow-hidden">
        {/* Poster Image */}
        {content.image_url ? (
          <Image
            src={content.image_url}
            alt={content.title}
            fill
            className="object-cover transition-transform duration-500 group-hover:scale-105"
            sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
            onError={(e) => {
              // Hide broken images
              (e.target as HTMLImageElement).style.display = 'none';
            }}
          />
        ) : (
          /* Fallback decorative pattern */
          <div className="absolute inset-0 opacity-20">
            <div className="absolute top-4 left-4 w-20 h-20 rounded-full bg-emerald-500/30 blur-2xl" />
            <div className="absolute bottom-4 right-4 w-16 h-16 rounded-full bg-blue-500/30 blur-2xl" />
          </div>
        )}

        {/* Content type badge */}
        <div className="absolute top-3 right-3">
          <span className={`px-3 py-1 rounded-full text-xs font-medium border ${getTypeStyle(content.content_type)}`}>
            {content.content_type === 'series' ? 'Series' : 'Movie'}
          </span>
        </div>

        {/* Year and language badge */}
        <div className="absolute bottom-3 left-3 flex items-center gap-2">
          <span className="px-2 py-1 rounded-lg bg-black/50 backdrop-blur-sm text-xs text-zinc-300">
            {content.year}
          </span>
          <span className="px-2 py-1 rounded-lg bg-black/50 backdrop-blur-sm text-xs text-zinc-400 uppercase">
            {content.original_language || 'EN'}
          </span>
        </div>

        {/* Rating badge */}
        {content.rating && (
          <div className="absolute bottom-3 right-3 flex items-center gap-1 px-2 py-1 rounded-lg bg-black/50 backdrop-blur-sm">
            <svg className="w-3.5 h-3.5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
              <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
            </svg>
            <span className="text-xs font-medium text-yellow-400">{content.rating}</span>
          </div>
        )}

        {/* Gradient overlay */}
        <div className="absolute inset-0 bg-gradient-to-t from-zinc-900 via-transparent to-transparent" />
      </div>

      {/* Content body */}
      <div className="p-4 space-y-3">
        {/* Title and network */}
        <div>
          <h3 className="text-lg font-semibold text-white line-clamp-1 group-hover:text-emerald-400 transition-colors">
            {content.title}
          </h3>
          {content.network_name && (
            <p className="text-xs text-zinc-500 mt-0.5 flex items-center gap-1">
              <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              {content.network_name}
            </p>
          )}
        </div>

        {/* Overview */}
        <p className="text-sm text-zinc-400 line-clamp-2 leading-relaxed">
          {content.overview || 'No description available.'}
        </p>

        {/* Genre tags */}
        <div className="flex flex-wrap gap-1.5">
          {content.genres?.slice(0, 3).map((genre) => (
            <span
              key={genre}
              className="px-2 py-0.5 rounded-md text-xs font-medium bg-zinc-800/80 text-zinc-400 border border-zinc-700/50"
            >
              {genre}
            </span>
          ))}
          {content.genres && content.genres.length > 3 && (
            <span className="px-2 py-0.5 rounded-md text-xs font-medium bg-zinc-800/80 text-zinc-500 border border-zinc-700/50">
              +{content.genres.length - 3}
            </span>
          )}
        </div>

        {/* Similarity match indicator */}
        {content.similarity !== undefined && (
          <div className="flex items-center gap-3 py-2">
            <div className="flex-1 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
              <div
                className="h-full learning-bar rounded-full transition-all duration-700"
                style={{ width: `${content.similarity * 100}%` }}
              />
            </div>
            <span className="text-xs font-medium text-emerald-400 min-w-[60px] text-right">
              {(content.similarity * 100).toFixed(0)}% match
            </span>
          </div>
        )}
      </div>
      </Link>

      {/* Action footer */}
      <div className="divider-glow" />
      <div className="flex items-center justify-between p-3 gap-2">
        <button
          onClick={handleSkip}
          disabled={actionState !== 'idle'}
          className={`
            flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl text-sm font-medium
            transition-all
            ${actionState === 'idle'
              ? 'text-zinc-400 hover:text-red-400 hover:bg-red-500/10 border border-transparent hover:border-red-500/20'
              : 'text-zinc-600 cursor-not-allowed'
            }
          `}
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
          Skip
        </button>

        <button
          onClick={() => onViewSimilar?.(content.id)}
          className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl text-sm font-medium text-zinc-400 hover:text-blue-400 hover:bg-blue-500/10 border border-transparent hover:border-blue-500/20 transition-all"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
          </svg>
          Similar
        </button>

        <button
          onClick={handleWatch}
          disabled={actionState !== 'idle'}
          className={`
            flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl text-sm font-medium
            transition-all group/btn
            ${actionState === 'idle'
              ? 'text-white bg-emerald-500/20 hover:bg-emerald-500/30 border border-emerald-500/30 hover:border-emerald-500/50'
              : actionState === 'liked'
              ? 'text-emerald-400 bg-emerald-500/30 border border-emerald-500/50'
              : 'text-zinc-600 bg-zinc-800/50 border border-zinc-700/50 cursor-not-allowed'
            }
          `}
        >
          {actionState === 'liked' ? (
            <>
              <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              <span className="text-emerald-400">Added!</span>
            </>
          ) : (
            <>
              <svg className="w-4 h-4 text-emerald-400 group-hover/btn:scale-110 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span className="text-emerald-400">Watch</span>
            </>
          )}
        </button>
      </div>
    </div>
  );
}
