'use client';

import { motion } from 'framer-motion';
import { ReactNode } from 'react';

interface AuthFormProps {
  children: ReactNode;
  onSubmit: (e: React.FormEvent) => void;
  title: string;
  subtitle?: string;
}

/**
 * Reusable auth form wrapper with styling and animations
 */
export function AuthForm({ children, onSubmit, title, subtitle }: AuthFormProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className="w-full max-w-md"
    >
      <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8">
        {/* Header */}
        <div className="mb-8 text-center">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            {title}
          </h1>
          {subtitle && (
            <p className="text-gray-600 dark:text-gray-400">{subtitle}</p>
          )}
        </div>

        {/* Form */}
        <form onSubmit={onSubmit} className="space-y-6">
          {children}
        </form>
      </div>
    </motion.div>
  );
}

interface FormFieldProps {
  label: string;
  error?: string;
  children: ReactNode;
}

/**
 * Form field wrapper with label and error display
 */
export function FormField({ label, error, children }: FormFieldProps) {
  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
        {label}
      </label>
      {children}
      {error && (
        <motion.p
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-sm text-red-500"
        >
          {error}
        </motion.p>
      )}
    </div>
  );
}

interface FormInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
}

/**
 * Styled form input
 */
export function FormInput({ error, className = '', ...props }: FormInputProps) {
  return (
    <input
      className={`
        w-full px-4 py-3 rounded-lg border
        ${
          error
            ? 'border-red-500 focus:ring-red-500'
            : 'border-gray-300 dark:border-gray-600 focus:ring-blue-500'
        }
        focus:ring-2 focus:border-transparent
        bg-white dark:bg-gray-700
        text-gray-900 dark:text-white
        placeholder-gray-500 dark:placeholder-gray-400
        transition-all duration-200
        ${className}
      `}
      {...props}
    />
  );
}
