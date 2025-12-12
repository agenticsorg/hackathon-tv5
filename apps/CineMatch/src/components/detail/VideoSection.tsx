'use client';

import type { Video } from '@/lib/tmdb';

interface VideoSectionProps {
  videos: Video[];
}

export function VideoSection({ videos }: VideoSectionProps) {
  // Filter to show trailers and teasers from YouTube
  const youtubeVideos = videos.filter((v) => v.site === 'YouTube');
  const trailers = youtubeVideos.filter((v) => v.type === 'Trailer');
  const teasers = youtubeVideos.filter((v) => v.type === 'Teaser');
  const other = youtubeVideos.filter(
    (v) => v.type !== 'Trailer' && v.type !== 'Teaser'
  );

  const displayVideos = [...trailers, ...teasers, ...other].slice(0, 6);

  if (!displayVideos.length) return null;

  return (
    <section className="py-8 px-4 md:px-8">
      <div className="max-w-7xl mx-auto">
        <h2 className="text-xl md:text-2xl font-bold text-white mb-6">
          Videos
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {displayVideos.map((video) => (
            <a
              key={video.id}
              href={`https://www.youtube.com/watch?v=${video.key}`}
              target="_blank"
              rel="noopener noreferrer"
              className="group relative aspect-video rounded-lg overflow-hidden bg-gray-800"
            >
              {/* YouTube thumbnail */}
              <img
                src={`https://img.youtube.com/vi/${video.key}/mqdefault.jpg`}
                alt={video.name}
                className="absolute inset-0 w-full h-full object-cover transition-transform group-hover:scale-105"
              />

              {/* Overlay */}
              <div className="absolute inset-0 bg-black/40 group-hover:bg-black/30 transition-colors" />

              {/* Play button */}
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="w-14 h-14 rounded-full bg-red-600 flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform">
                  <svg
                    className="w-6 h-6 text-white ml-1"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path d="M6.3 2.841A1.5 1.5 0 004 4.11v11.78a1.5 1.5 0 002.3 1.269l9.344-5.89a1.5 1.5 0 000-2.538L6.3 2.84z" />
                  </svg>
                </div>
              </div>

              {/* Video info */}
              <div className="absolute bottom-0 left-0 right-0 p-3">
                <span className="inline-block px-2 py-0.5 bg-gray-900/80 text-gray-300 text-xs rounded mb-1">
                  {video.type}
                </span>
                <p className="text-white text-sm font-medium line-clamp-1">
                  {video.name}
                </p>
              </div>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}
