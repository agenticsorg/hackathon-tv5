'use client';

import { useEffect, useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Image from 'next/image';
import Link from 'next/link';
import type { Content } from '@/lib/db';

interface SimilarContent extends Content {
  similarity: number;
}

export default function ContentDetailPage() {
  const params = useParams();
  const router = useRouter();
  const [content, setContent] = useState<Content | null>(null);
  const [similar, setSimilar] = useState<SimilarContent[]>([]);
  const [loading, setLoading] = useState(true);
  const [actionState, setActionState] = useState<'idle' | 'liked' | 'skipped'>('idle');

  useEffect(() => {
    const fetchContent = async () => {
      try {
        // Fetch main content details
        const response = await fetch(`/api/content/${params.id}`);
        if (!response.ok) throw new Error('Content not found');
        const data = await response.json();
        setContent(data.content);

        // Fetch similar content
        const similarResponse = await fetch(`/api/similar?id=${params.id}&limit=6`);
        const similarData = await similarResponse.json();
        setSimilar(similarData.results || []);
      } catch (error) {
        console.error('Failed to fetch content:', error);
      } finally {
        setLoading(false);
      }
    };

    if (params.id) {
      fetchContent();
    }
  }, [params.id]);

  const handleWatch = async () => {
    if (!content) return;
    setActionState('liked');
    try {
      await fetch('/api/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ contentId: content.id, wasSuccessful: true }),
      });
    } catch (error) {
      console.error('Failed to record feedback:', error);
    }
  };

  const handleSkip = async () => {
    if (!content) return;
    setActionState('skipped');
    try {
      await fetch('/api/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ contentId: content.id, wasSuccessful: false }),
      });
    } catch (error) {
      console.error('Failed to record feedback:', error);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-zinc-950 via-zinc-900 to-zinc-950 flex items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="w-12 h-12 border-4 border-emerald-500/30 border-t-emerald-500 rounded-full animate-spin" />
          <p className="text-zinc-400">Loading content...</p>
        </div>
      </div>
    );
  }

  if (!content) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-zinc-950 via-zinc-900 to-zinc-950 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-white mb-4">Content Not Found</h1>
          <Link href="/" className="text-emerald-400 hover:underline">
            Return to Home
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-zinc-950 via-zinc-900 to-zinc-950">
      {/* Hero Section with Backdrop */}
      <div className="relative h-[50vh] md:h-[60vh] overflow-hidden">
        {/* Background Image with blur */}
        {content.image_url && (
          <div className="absolute inset-0">
            <Image
              src={content.image_url}
              alt=""
              fill
              className="object-cover opacity-30 blur-sm scale-110"
              priority
            />
          </div>
        )}

        {/* Gradient overlays */}
        <div className="absolute inset-0 bg-gradient-to-t from-zinc-950 via-zinc-950/60 to-transparent" />
        <div className="absolute inset-0 bg-gradient-to-r from-zinc-950/80 via-transparent to-zinc-950/80" />

        {/* Back button */}
        <div className="absolute top-6 left-6 z-20">
          <button
            onClick={() => router.back()}
            className="flex items-center gap-2 px-4 py-2 rounded-xl bg-zinc-900/80 backdrop-blur-sm border border-zinc-700/50 text-zinc-300 hover:text-white hover:border-zinc-600 transition-all"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
            Back
          </button>
        </div>

        {/* Content Info */}
        <div className="absolute bottom-0 left-0 right-0 p-6 md:p-12">
          <div className="max-w-6xl mx-auto flex flex-col md:flex-row gap-8 items-end md:items-end">
            {/* Poster */}
            <div className="relative w-40 md:w-56 aspect-[2/3] rounded-2xl overflow-hidden shadow-2xl border border-zinc-700/50 flex-shrink-0 group">
              {content.image_url ? (
                <Image
                  src={content.image_url}
                  alt={content.title}
                  fill
                  className="object-cover transition-transform duration-500 group-hover:scale-105"
                  priority
                />
              ) : (
                <div className="absolute inset-0 bg-gradient-to-br from-zinc-800 to-zinc-900 flex items-center justify-center">
                  <svg className="w-16 h-16 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
                  </svg>
                </div>
              )}
            </div>

            {/* Title & Meta */}
            <div className="flex-1 space-y-4">
              {/* Badges */}
              <div className="flex flex-wrap items-center gap-2">
                <span className={`px-3 py-1 rounded-full text-xs font-medium border ${
                  content.content_type === 'series'
                    ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30'
                    : 'bg-blue-500/20 text-blue-400 border-blue-500/30'
                }`}>
                  {content.content_type === 'series' ? 'TV Series' : 'Movie'}
                </span>
                <span className="px-3 py-1 rounded-full text-xs font-medium bg-zinc-800/80 text-zinc-300 border border-zinc-700/50">
                  {content.year}
                </span>
                {content.original_language && (
                  <span className="px-3 py-1 rounded-full text-xs font-medium bg-zinc-800/80 text-zinc-400 border border-zinc-700/50 uppercase">
                    {content.original_language}
                  </span>
                )}
              </div>

              {/* Title */}
              <h1 className="text-3xl md:text-5xl font-bold text-white leading-tight">
                {content.title}
              </h1>

              {/* Network & Rating */}
              <div className="flex items-center gap-4 text-sm">
                {content.network_name && (
                  <div className="flex items-center gap-2 text-zinc-400">
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                    {content.network_name}
                  </div>
                )}
                {content.rating && (
                  <div className="flex items-center gap-1">
                    <svg className="w-5 h-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
                      <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                    </svg>
                    <span className="text-yellow-400 font-semibold">{content.rating}</span>
                    <span className="text-zinc-500">/ 10</span>
                  </div>
                )}
              </div>

              {/* Genres */}
              <div className="flex flex-wrap gap-2">
                {content.genres?.map((genre) => (
                  <span
                    key={genre}
                    className="px-3 py-1.5 rounded-lg text-sm font-medium bg-zinc-800/60 text-zinc-300 border border-zinc-700/50 hover:border-emerald-500/30 hover:text-emerald-400 transition-colors cursor-default"
                  >
                    {genre}
                  </span>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Content Body */}
      <div className="max-w-6xl mx-auto px-6 md:px-12 py-8 space-y-12">
        {/* Action Buttons */}
        <div className="flex flex-wrap gap-4">
          <button
            onClick={handleWatch}
            disabled={actionState !== 'idle'}
            className={`flex items-center gap-2 px-8 py-3 rounded-xl font-semibold transition-all ${
              actionState === 'liked'
                ? 'bg-emerald-500 text-white'
                : actionState === 'idle'
                ? 'bg-emerald-500 hover:bg-emerald-400 text-white shadow-lg shadow-emerald-500/25'
                : 'bg-zinc-700 text-zinc-400 cursor-not-allowed'
            }`}
          >
            {actionState === 'liked' ? (
              <>
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                Added to Watchlist
              </>
            ) : (
              <>
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Add to Watchlist
              </>
            )}
          </button>

          <button
            onClick={handleSkip}
            disabled={actionState !== 'idle'}
            className={`flex items-center gap-2 px-6 py-3 rounded-xl font-medium border transition-all ${
              actionState === 'skipped'
                ? 'bg-zinc-700 border-zinc-600 text-zinc-400'
                : actionState === 'idle'
                ? 'border-zinc-700 text-zinc-300 hover:border-red-500/50 hover:text-red-400 hover:bg-red-500/10'
                : 'border-zinc-800 text-zinc-600 cursor-not-allowed'
            }`}
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
            {actionState === 'skipped' ? 'Skipped' : 'Not Interested'}
          </button>
        </div>

        {/* Overview Section */}
        <div className="space-y-4">
          <h2 className="text-xl font-semibold text-white flex items-center gap-2">
            <svg className="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            Overview
          </h2>
          <p className="text-zinc-400 leading-relaxed text-lg max-w-4xl">
            {content.overview || 'No overview available for this title.'}
          </p>
        </div>

        {/* AI Learning Insight */}
        <div className="glass-card rounded-2xl p-6 border border-emerald-500/20">
          <div className="flex items-start gap-4">
            <div className="w-12 h-12 rounded-xl bg-emerald-500/20 flex items-center justify-center flex-shrink-0">
              <svg className="w-6 h-6 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
              </svg>
            </div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-white mb-1">AI Learning</h3>
              <p className="text-zinc-400 text-sm">
                Your feedback helps our Q-Learning algorithm improve recommendations.
                When you watch or skip content, we update our pattern recognition to better understand your preferences.
              </p>
            </div>
          </div>
        </div>

        {/* Similar Content Section */}
        {similar.length > 0 && (
          <div className="space-y-6">
            <h2 className="text-xl font-semibold text-white flex items-center gap-2">
              <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              Similar Titles
            </h2>
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
              {similar.map((item) => (
                <Link
                  key={item.id}
                  href={`/content/${item.id}`}
                  className="group"
                >
                  <div className="relative aspect-[2/3] rounded-xl overflow-hidden bg-zinc-800 border border-zinc-700/50 hover:border-emerald-500/50 transition-all">
                    {item.image_url ? (
                      <Image
                        src={item.image_url}
                        alt={item.title}
                        fill
                        className="object-cover transition-transform duration-300 group-hover:scale-105"
                      />
                    ) : (
                      <div className="absolute inset-0 flex items-center justify-center">
                        <svg className="w-8 h-8 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
                        </svg>
                      </div>
                    )}
                    {/* Similarity badge */}
                    {item.similarity && (
                      <div className="absolute top-2 right-2 px-2 py-0.5 rounded-full bg-emerald-500/90 text-xs font-medium text-white">
                        {(item.similarity * 100).toFixed(0)}%
                      </div>
                    )}
                    {/* Gradient overlay */}
                    <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                    {/* Title on hover */}
                    <div className="absolute bottom-0 left-0 right-0 p-3 opacity-0 group-hover:opacity-100 transition-opacity">
                      <p className="text-white text-sm font-medium line-clamp-2">{item.title}</p>
                      <p className="text-zinc-400 text-xs mt-1">{item.year}</p>
                    </div>
                  </div>
                </Link>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
