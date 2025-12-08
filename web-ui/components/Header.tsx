'use client';

import { useState } from 'react';

export default function Header() {
  return (
    <header className="bg-bg-dark border-b border-bg-dark-border sticky top-0 z-50">
      <div className="max-w-[1400px] mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <div className="flex items-center space-x-2">
            <div className="w-10 h-10 bg-youtube-red rounded-lg flex items-center justify-center">
              <svg className="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
              </svg>
            </div>
            <div>
              <h1 className="text-xl font-bold text-text-dark-primary">EntertainAI</h1>
              <p className="text-xs text-text-dark-secondary">8-Agent Discovery System</p>
            </div>
          </div>

          {/* Navigation */}
          <nav className="hidden md:flex items-center space-x-6">
            <a href="#discover" className="text-text-dark-secondary hover:text-text-dark-primary transition-colors">
              Discover
            </a>
            <a href="#trending" className="text-text-dark-secondary hover:text-text-dark-primary transition-colors">
              Trending
            </a>
            <a href="#my-list" className="text-text-dark-secondary hover:text-text-dark-primary transition-colors">
              My List
            </a>
          </nav>

          {/* User Menu */}
          <div className="flex items-center space-x-4">
            <button
              className="w-10 h-10 rounded-full bg-youtube-red flex items-center justify-center hover:bg-youtube-red-dark transition-colors"
              aria-label="User profile"
            >
              <svg className="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clipRule="evenodd" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </header>
  );
}
