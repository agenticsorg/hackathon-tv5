import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import { Providers } from './providers';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'TVDB Smart Recommendations | AI-Powered Entertainment Discovery',
  description: 'Self-learning recommendation system that solves the 45-minute decision problem. Find what to watch in seconds, not minutes.',
  keywords: ['TV recommendations', 'AI', 'entertainment discovery', 'TVDB', 'movies', 'TV shows'],
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.className} gradient-bg min-h-screen`}>
        <Providers>
          {children}
        </Providers>
      </body>
    </html>
  );
}
