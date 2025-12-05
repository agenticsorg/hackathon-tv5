'use client';

import Image from 'next/image';
import type { CastMember } from '@/lib/tmdb';

const PROFILE_BASE = 'https://image.tmdb.org/t/p/w185';

interface CastSectionProps {
  cast: CastMember[];
}

export function CastSection({ cast }: CastSectionProps) {
  if (!cast.length) return null;

  return (
    <section className="py-8 px-4 md:px-8">
      <div className="max-w-7xl mx-auto">
        <h2 className="text-xl md:text-2xl font-bold text-white mb-6">Cast</h2>

        <div className="relative">
          <div className="flex gap-4 overflow-x-auto pb-4 scrollbar-hide">
            {cast.map((member) => (
              <div
                key={member.id}
                className="flex-shrink-0 w-28 md:w-36 group"
              >
                <div className="relative aspect-[2/3] rounded-lg overflow-hidden bg-gray-800 mb-2">
                  {member.profilePath ? (
                    <Image
                      src={`${PROFILE_BASE}${member.profilePath}`}
                      alt={member.name}
                      fill
                      className="object-cover transition-transform group-hover:scale-105"
                      sizes="(max-width: 768px) 112px, 144px"
                    />
                  ) : (
                    <div className="absolute inset-0 flex items-center justify-center text-gray-600">
                      <svg
                        className="w-10 h-10"
                        fill="currentColor"
                        viewBox="0 0 20 20"
                      >
                        <path
                          fillRule="evenodd"
                          d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z"
                          clipRule="evenodd"
                        />
                      </svg>
                    </div>
                  )}
                </div>
                <p className="font-medium text-white text-sm line-clamp-1">
                  {member.name}
                </p>
                <p className="text-gray-400 text-xs line-clamp-1">
                  {member.character}
                </p>
              </div>
            ))}
          </div>

          {/* Fade edges */}
          <div className="absolute top-0 right-0 bottom-4 w-12 bg-gradient-to-l from-gray-950 to-transparent pointer-events-none" />
        </div>
      </div>
    </section>
  );
}
