'use client';

import { motion, HTMLMotionProps } from 'framer-motion';
import { Loader2 } from 'lucide-react';

interface LoadingButtonProps {
  loading?: boolean;
  children: React.ReactNode;
  variant?: 'primary' | 'secondary' | 'outline';
  className?: string;
  type?: 'button' | 'submit' | 'reset';
  onClick?: () => void;
  disabled?: boolean;
}

/**
 * Button with loading spinner
 */
export function LoadingButton({
  loading = false,
  children,
  variant = 'primary',
  className = '',
  disabled,
  type = 'button',
  onClick,
}: LoadingButtonProps) {
  const baseStyles = `
    w-full px-6 py-3 rounded-lg font-medium
    transition-all duration-200
    focus:ring-2 focus:ring-offset-2
    disabled:opacity-50 disabled:cursor-not-allowed
    flex items-center justify-center gap-2
  `;

  const variantStyles = {
    primary: `
      bg-gradient-to-r from-blue-500 to-purple-600
      hover:from-blue-600 hover:to-purple-700
      text-white
      focus:ring-blue-500
      shadow-lg hover:shadow-xl
    `,
    secondary: `
      bg-gray-200 dark:bg-gray-700
      hover:bg-gray-300 dark:hover:bg-gray-600
      text-gray-900 dark:text-white
      focus:ring-gray-500
    `,
    outline: `
      border-2 border-gray-300 dark:border-gray-600
      hover:border-blue-500 dark:hover:border-blue-500
      text-gray-900 dark:text-white
      focus:ring-blue-500
    `,
  };

  return (
    <motion.button
      whileHover={{ scale: disabled || loading ? 1 : 1.02 }}
      whileTap={{ scale: disabled || loading ? 1 : 0.98 }}
      className={`${baseStyles} ${variantStyles[variant]} ${className}`}
      disabled={disabled || loading}
      type={type}
      onClick={onClick}
    >
      {loading && (
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
        >
          <Loader2 size={20} />
        </motion.div>
      )}
      {children}
    </motion.button>
  );
}
