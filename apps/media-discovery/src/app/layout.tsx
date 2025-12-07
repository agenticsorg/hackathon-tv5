import type { Metadata } from 'next';
import './globals.css';
import { Providers } from './providers';

export const metadata: Metadata = {
  title: 'AI Media Discovery',
  description: 'Discover movies and TV shows through natural language prompts and personalized recommendations',
  openGraph: {
    title: 'AI Media Discovery',
    description: 'Find your next favorite movie or show with natural language search',
    type: 'website',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        {/* Fonts */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Funnel+Display:wght@300..800&display=swap"
          rel="stylesheet"
        />
        {/* ARW Discovery */}
        <link
          rel="alternate"
          type="application/json"
          href="/.well-known/arw-manifest.json"
        />
        <link
          rel="alternate"
          type="text/plain"
          href="/llms.txt"
        />
      </head>
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
