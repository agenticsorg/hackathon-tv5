import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../stores/appStore';
import './MarkdownPreview.css';

interface MarkdownPreviewProps {
  content: string;
}

export function MarkdownPreview({ content }: MarkdownPreviewProps) {
  const [html, setHtml] = useState('');
  const { openNote, settings } = useAppStore();

  useEffect(() => {
    const renderContent = async () => {
      try {
        const rendered = await invoke<string>('render_markdown', { content });
        setHtml(rendered);
      } catch (error) {
        // Fallback: basic HTML escaping
        const escaped = content
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
          .replace(/\n/g, '<br>');
        setHtml(`<pre>${escaped}</pre>`);
      }
    };

    renderContent();
  }, [content]);

  // Handle link clicks
  const handleClick = (e: React.MouseEvent) => {
    const target = e.target as HTMLElement;
    const link = target.closest('a');

    if (link) {
      e.preventDefault();
      const href = link.getAttribute('href');

      if (href) {
        // Internal wiki link
        if (href.startsWith('obsidian://')) {
          const path = href.replace('obsidian://', '');
          openNote(path);
        }
        // External link
        else if (href.startsWith('http://') || href.startsWith('https://')) {
          window.open(href, '_blank');
        }
        // Relative path
        else {
          openNote(href);
        }
      }
    }
  };

  return (
    <div
      className="markdown-preview"
      style={{
        '--preview-font-size': `${settings.fontSize}px`,
        '--preview-line-width': `${settings.lineWidth}px`,
      } as React.CSSProperties}
      onClick={handleClick}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
