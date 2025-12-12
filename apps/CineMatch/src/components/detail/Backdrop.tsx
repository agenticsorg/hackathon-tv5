'use client';

import Image from 'next/image';
import { getBackdropUrl } from '@/lib/tmdb';

interface BackdropProps {
  path: string | null;
  title: string;
}

export function Backdrop({ path, title }: BackdropProps) {
  const backdropUrl = getBackdropUrl(path);

  return (
    <div className="relative h-[50vh] md:h-[60vh] w-full overflow-hidden">
      {backdropUrl ? (
        <>
          <Image
            src={backdropUrl}
            alt={`${title} backdrop`}
            fill
            priority
            className="object-cover"
            sizes="100vw"
          />
          {/* Gradient overlays */}
          <div className="absolute inset-0 bg-gradient-to-t from-gray-950 via-gray-950/60 to-transparent" />
          <div className="absolute inset-0 bg-gradient-to-r from-gray-950/80 via-transparent to-transparent" />
        </>
      ) : (
        <div className="absolute inset-0 bg-gradient-to-b from-gray-800 to-gray-950" />
      )}
    </div>
  );
}
