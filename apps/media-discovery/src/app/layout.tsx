import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import { Providers } from './providers';

const inter = Inter({ subsets: ['latin'] });

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
      <body className={inter.className}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
