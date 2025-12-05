'use client';

import { use } from 'react';
import { useQuery } from '@tanstack/react-query';
import Link from 'next/link';
import { Backdrop } from '@/components/detail/Backdrop';
import { DetailHero } from '@/components/detail/DetailHero';
import { CastSection } from '@/components/detail/CastSection';
import { VideoSection } from '@/components/detail/VideoSection';
import { RelatedSection } from '@/components/detail/RelatedSection';
import type { Movie, Genre } from '@/types/media';
import type { CastMember, CrewMember, Video } from '@/lib/tmdb';

interface MovieDetailsResponse {
  success: boolean;
  movie: Movie;
  credits: { cast: CastMember[]; crew: CrewMember[] };
  videos: Video[];
  similar: Movie[];
  recommendations: Movie[];
  genres: Genre[];
}

async function fetchMovieDetails(id: number): Promise<MovieDetailsResponse> {
  const response = await fetch(`/api/movies/${id}`);
  if (!response.ok) throw new Error('Failed to fetch movie details');
  return response.json();
}

export default function MoviePage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  const movieId = parseInt(id, 10);

  const { data, isLoading, error } = useQuery({
    queryKey: ['movie', movieId],
    queryFn: () => fetchMovieDetails(movieId),
    enabled: !isNaN(movieId),
  });

  if (isLoading) {
    return <MovieSkeleton />;
  }

  if (error || !data?.success) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-950">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-white mb-2">Movie Not Found</h1>
          <p className="text-gray-400 mb-4">The movie you&apos;re looking for doesn&apos;t exist.</p>
          <Link
            href="/"
            className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
            Back to Home
          </Link>
        </div>
      </div>
    );
  }

  const { movie, credits, videos, similar, recommendations, genres } = data;

  return (
    <main className="min-h-screen bg-gray-950">
      {/* Back button */}
      <div className="fixed top-4 left-4 z-50">
        <Link
          href="/"
          className="flex items-center justify-center w-10 h-10 bg-gray-900/80 hover:bg-gray-800 backdrop-blur-sm rounded-full transition-colors"
        >
          <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </Link>
      </div>

      {/* Backdrop */}
      <Backdrop path={movie.backdropPath} title={movie.title} />

      {/* Hero content */}
      <DetailHero content={movie} genres={genres} videos={videos} />

      {/* Cast */}
      <CastSection cast={credits.cast} />

      {/* Videos */}
      <VideoSection videos={videos} />

      {/* Related content */}
      <RelatedSection similar={similar} recommendations={recommendations} />

      {/* Footer spacing */}
      <div className="h-16" />
    </main>
  );
}

function MovieSkeleton() {
  return (
    <div className="min-h-screen bg-gray-950">
      {/* Backdrop skeleton */}
      <div className="relative h-[50vh] md:h-[60vh] bg-gray-900 animate-pulse" />

      {/* Hero skeleton */}
      <div className="relative z-10 -mt-48 md:-mt-64 px-4 md:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="flex flex-col md:flex-row gap-6 md:gap-10">
            {/* Poster skeleton */}
            <div className="flex-shrink-0 w-48 md:w-64 mx-auto md:mx-0">
              <div className="aspect-[2/3] rounded-xl bg-gray-800 animate-pulse" />
            </div>

            {/* Content skeleton */}
            <div className="flex-1 space-y-4">
              <div className="h-10 w-3/4 bg-gray-800 rounded animate-pulse mx-auto md:mx-0" />
              <div className="h-6 w-1/2 bg-gray-800 rounded animate-pulse mx-auto md:mx-0" />
              <div className="flex gap-2 justify-center md:justify-start">
                {[1, 2, 3].map((i) => (
                  <div key={i} className="h-8 w-20 bg-gray-800 rounded-full animate-pulse" />
                ))}
              </div>
              <div className="space-y-2">
                <div className="h-4 bg-gray-800 rounded animate-pulse" />
                <div className="h-4 bg-gray-800 rounded animate-pulse" />
                <div className="h-4 w-2/3 bg-gray-800 rounded animate-pulse" />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Cast skeleton */}
      <div className="py-8 px-4 md:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="h-8 w-24 bg-gray-800 rounded animate-pulse mb-6" />
          <div className="flex gap-4">
            {[1, 2, 3, 4, 5, 6].map((i) => (
              <div key={i} className="flex-shrink-0 w-28 md:w-36">
                <div className="aspect-[2/3] rounded-lg bg-gray-800 animate-pulse mb-2" />
                <div className="h-4 bg-gray-800 rounded animate-pulse mb-1" />
                <div className="h-3 w-3/4 bg-gray-800 rounded animate-pulse" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
