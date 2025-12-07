import { useEffect, useRef, useCallback } from 'react';
import { EditorView, keymap, lineNumbers, highlightActiveLine, drawSelection, highlightActiveLineGutter } from '@codemirror/view';
import { EditorState, Compartment } from '@codemirror/state';
import { markdown } from '@codemirror/lang-markdown';
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from '@codemirror/language';
import { useAppStore } from '../stores/appStore';
import './MarkdownEditor.css';

interface MarkdownEditorProps {
  content: string;
  onChange: (content: string) => void;
  onSave: () => void;
}

const themeCompartment = new Compartment();
const lineNumbersCompartment = new Compartment();

function createObsidianTheme(isDark: boolean) {
  return EditorView.theme({
    '&': {
      height: '100%',
      fontSize: 'var(--editor-font-size)',
    },
    '.cm-content': {
      fontFamily: 'var(--font-monospace)',
      padding: '16px 0',
      maxWidth: 'var(--editor-line-width)',
      margin: '0 auto',
    },
    '.cm-line': {
      padding: '0 16px',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--background-primary)',
      color: 'var(--text-muted)',
      border: 'none',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'var(--background-secondary)',
    },
    '.cm-activeLine': {
      backgroundColor: 'var(--background-secondary-alt)',
    },
    '.cm-selectionBackground': {
      backgroundColor: 'var(--text-selection)',
    },
    '&.cm-focused .cm-selectionBackground': {
      backgroundColor: 'var(--text-selection)',
    },
    '.cm-cursor': {
      borderLeftColor: 'var(--text-accent)',
    },
    // Markdown syntax highlighting
    '.cm-header': {
      color: 'var(--text-accent)',
      fontWeight: '600',
    },
    '.cm-header-1': {
      fontSize: '1.6em',
    },
    '.cm-header-2': {
      fontSize: '1.4em',
    },
    '.cm-header-3': {
      fontSize: '1.2em',
    },
    '.cm-strong': {
      fontWeight: '700',
      color: 'var(--text-normal)',
    },
    '.cm-em': {
      fontStyle: 'italic',
      color: 'var(--text-normal)',
    },
    '.cm-link': {
      color: 'var(--text-accent)',
      textDecoration: 'underline',
    },
    '.cm-url': {
      color: 'var(--text-muted)',
    },
    '.cm-quote': {
      color: 'var(--text-muted)',
      fontStyle: 'italic',
    },
    '.cm-list': {
      color: 'var(--text-accent)',
    },
    '.cm-hr': {
      color: 'var(--text-muted)',
    },
    '.cm-comment': {
      color: 'var(--text-faint)',
    },
    '.cm-meta': {
      color: 'var(--text-muted)',
    },
  }, { dark: isDark });
}

export function MarkdownEditor({ content, onChange, onSave }: MarkdownEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const { settings } = useAppStore();

  const handleChange = useCallback((update: { docChanged: boolean; state: EditorState }) => {
    if (update.docChanged) {
      const newContent = update.state.doc.toString();
      onChange(newContent);
    }
  }, [onChange]);

  useEffect(() => {
    if (!editorRef.current) return;

    const isDark = settings.theme === 'dark' ||
      (settings.theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

    const extensions = [
      EditorView.updateListener.of(handleChange),
      markdown(),
      history(),
      bracketMatching(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      drawSelection(),
      syntaxHighlighting(defaultHighlightStyle),
      keymap.of([
        ...defaultKeymap,
        ...historyKeymap,
        indentWithTab,
        {
          key: 'Mod-s',
          run: () => {
            onSave();
            return true;
          },
        },
      ]),
      themeCompartment.of(createObsidianTheme(isDark)),
      lineNumbersCompartment.of(settings.showLineNumbers ? lineNumbers() : []),
      EditorView.lineWrapping,
    ];

    const state = EditorState.create({
      doc: content,
      extensions,
    });

    const view = new EditorView({
      state,
      parent: editorRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, []); // Only run once on mount

  // Update content when it changes externally
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;

    const currentContent = view.state.doc.toString();
    if (content !== currentContent) {
      view.dispatch({
        changes: {
          from: 0,
          to: currentContent.length,
          insert: content,
        },
      });
    }
  }, [content]);

  // Update theme when settings change
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;

    const isDark = settings.theme === 'dark' ||
      (settings.theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

    view.dispatch({
      effects: [
        themeCompartment.reconfigure(createObsidianTheme(isDark)),
        lineNumbersCompartment.reconfigure(settings.showLineNumbers ? lineNumbers() : []),
      ],
    });
  }, [settings.theme, settings.showLineNumbers]);

  return (
    <div
      ref={editorRef}
      className="markdown-editor"
      style={{
        '--editor-font-size': `${settings.fontSize}px`,
        '--editor-line-width': `${settings.lineWidth}px`,
      } as React.CSSProperties}
    />
  );
}
