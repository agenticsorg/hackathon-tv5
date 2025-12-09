/**
 * Simple logger utility
 * Production apps should use structured logging (e.g., Winston, Pino)
 */

export const logger = {
  info: (message: string, meta?: any) => {
    console.log(`[INFO] ${message}`, meta || '');
  },

  warn: (message: string, meta?: any) => {
    console.warn(`[WARN] ${message}`, meta || '');
  },

  error: (message: string, meta?: any) => {
    console.error(`[ERROR] ${message}`, meta || '');
  },

  debug: (message: string, meta?: any) => {
    if (process.env.NODE_ENV !== 'production') {
      console.debug(`[DEBUG] ${message}`, meta || '');
    }
  },
};
