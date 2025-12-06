'use client';

import { useState, forwardRef } from 'react';
import { Eye, EyeOff } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  calculatePasswordStrength,
  getPasswordStrengthLabel,
} from '../../lib/utils/validators';

interface PasswordInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
  showStrength?: boolean;
}

/**
 * Password input with visibility toggle and strength indicator
 */
export const PasswordInput = forwardRef<HTMLInputElement, PasswordInputProps>(
  ({ error, showStrength = false, value, ...props }, ref) => {
    const [showPassword, setShowPassword] = useState(false);

    const strength = showStrength && value
      ? calculatePasswordStrength(value as string)
      : 0;

    const { label, color } = getPasswordStrengthLabel(strength);

    return (
      <div className="space-y-2">
        <div className="relative">
          <input
            ref={ref}
            type={showPassword ? 'text' : 'password'}
            value={value}
            className={`
              w-full px-4 py-3 pr-12 rounded-lg border
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
            `}
            {...props}
          />
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
            tabIndex={-1}
          >
            {showPassword ? <EyeOff size={20} /> : <Eye size={20} />}
          </button>
        </div>

        {/* Password strength indicator */}
        <AnimatePresence>
          {showStrength && value && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              className="space-y-2"
            >
              {/* Strength bar */}
              <div className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${strength}%` }}
                  transition={{ duration: 0.3 }}
                  className={`h-full rounded-full ${
                    strength < 30
                      ? 'bg-red-500'
                      : strength < 60
                      ? 'bg-orange-500'
                      : strength < 80
                      ? 'bg-yellow-500'
                      : 'bg-green-500'
                  }`}
                />
              </div>

              {/* Strength label */}
              <p className={`text-sm font-medium ${color}`}>
                Password strength: {label}
              </p>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    );
  }
);

PasswordInput.displayName = 'PasswordInput';
