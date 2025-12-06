'use client';

import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { motion, AnimatePresence } from 'framer-motion';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { CheckCircle2, Sparkles } from 'lucide-react';
import { useRegister } from '../../../lib/hooks/use-auth';
import { useAuthStore } from '../../../lib/stores/auth-store';
import { registerSchema, type RegisterFormData } from '../../../lib/utils/validators';
import { AuthForm, FormField, FormInput } from '../../../components/shared/auth-form';
import { PasswordInput } from '../../../components/shared/password-input';
import { LoadingButton } from '../../../components/shared/loading-button';

/**
 * Register page with form validation, password strength, and success animation
 */
export default function RegisterPage() {
  const router = useRouter();
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const { mutate: register, isPending, isSuccess, error } = useRegister();
  const [showSuccess, setShowSuccess] = useState(false);

  const {
    register: registerField,
    handleSubmit,
    formState: { errors },
    watch,
  } = useForm<RegisterFormData>({
    resolver: zodResolver(registerSchema),
    defaultValues: {
      name: '',
      email: '',
      password: '',
      confirmPassword: '',
    },
  });

  const passwordValue = watch('password');
  const confirmPasswordValue = watch('confirmPassword');

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      router.push('/dashboard');
    }
  }, [isAuthenticated, router]);

  // Show success animation
  useEffect(() => {
    if (isSuccess) {
      setShowSuccess(true);
    }
  }, [isSuccess]);

  const onSubmit = (data: RegisterFormData) => {
    register({
      name: data.name,
      email: data.email,
      password: data.password,
    });
  };

  // Extract error message
  const errorMessage = error
    ? (error as any)?.response?.data?.message || 'Registration failed. Please try again.'
    : null;

  return (
    <>
      <AuthForm
        title="Create Account"
        subtitle="Start your personalized content journey"
        onSubmit={handleSubmit(onSubmit)}
      >
        {/* Success animation overlay */}
        <AnimatePresence>
          {showSuccess && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
            >
              <motion.div
                initial={{ scale: 0, rotate: -180 }}
                animate={{ scale: 1, rotate: 0 }}
                transition={{ type: 'spring', duration: 0.6 }}
                className="bg-white dark:bg-gray-800 rounded-2xl p-8 text-center max-w-md mx-4"
              >
                <motion.div
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{ delay: 0.2 }}
                  className="w-20 h-20 mx-auto mb-4 bg-green-100 dark:bg-green-900/20 rounded-full flex items-center justify-center"
                >
                  <CheckCircle2 className="w-12 h-12 text-green-600 dark:text-green-400" />
                </motion.div>
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                  Welcome to EmotiStream!
                </h2>
                <p className="text-gray-600 dark:text-gray-400 mb-4">
                  Your account has been created successfully. Redirecting to dashboard...
                </p>
                <motion.div
                  animate={{ rotate: 360 }}
                  transition={{ duration: 2, repeat: Infinity, ease: 'linear' }}
                  className="inline-block"
                >
                  <Sparkles className="w-6 h-6 text-purple-600" />
                </motion.div>
              </motion.div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Error message */}
        {errorMessage && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            className="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg"
          >
            <p className="text-sm text-red-600 dark:text-red-400">{errorMessage}</p>
          </motion.div>
        )}

        {/* Name field */}
        <FormField label="Full Name" error={errors.name?.message}>
          <FormInput
            type="text"
            placeholder="John Doe"
            error={!!errors.name}
            {...registerField('name')}
          />
        </FormField>

        {/* Email field */}
        <FormField label="Email" error={errors.email?.message}>
          <FormInput
            type="email"
            placeholder="you@example.com"
            error={!!errors.email}
            {...registerField('email')}
          />
        </FormField>

        {/* Password field with strength indicator */}
        <FormField label="Password" error={errors.password?.message}>
          <PasswordInput
            placeholder="Create a strong password"
            error={!!errors.password}
            showStrength
            value={passwordValue}
            {...registerField('password')}
          />
        </FormField>

        {/* Confirm password field */}
        <FormField label="Confirm Password" error={errors.confirmPassword?.message}>
          <PasswordInput
            placeholder="Confirm your password"
            error={!!errors.confirmPassword}
            value={confirmPasswordValue}
            {...registerField('confirmPassword')}
          />
        </FormField>

        {/* Terms and conditions */}
        <div className="text-xs text-gray-600 dark:text-gray-400 text-center">
          By creating an account, you agree to our{' '}
          <Link href="/terms" className="text-blue-600 hover:underline">
            Terms of Service
          </Link>{' '}
          and{' '}
          <Link href="/privacy" className="text-blue-600 hover:underline">
            Privacy Policy
          </Link>
        </div>

        {/* Submit button */}
        <LoadingButton type="submit" loading={isPending}>
          Create Account
        </LoadingButton>

        {/* Login link */}
        <div className="text-center">
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Already have an account?{' '}
            <Link
              href="/login"
              className="font-medium text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 transition-colors"
            >
              Sign in
            </Link>
          </p>
        </div>
      </AuthForm>
    </>
  );
}
