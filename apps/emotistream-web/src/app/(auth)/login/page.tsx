'use client';

import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { motion } from 'framer-motion';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useLogin } from '../../../lib/hooks/use-auth';
import { useAuthStore } from '../../../lib/stores/auth-store';
import { loginSchema, type LoginFormData } from '../../../lib/utils/validators';
import { AuthForm, FormField, FormInput } from '../../../components/shared/auth-form';
import { PasswordInput } from '../../../components/shared/password-input';
import { LoadingButton } from '../../../components/shared/loading-button';

/**
 * Login page with form validation and error handling
 */
export default function LoginPage() {
  const router = useRouter();
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const { mutate: login, isPending, error } = useLogin();

  const {
    register,
    handleSubmit,
    formState: { errors },
    watch,
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      email: '',
      password: '',
    },
  });

  const passwordValue = watch('password');

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      router.push('/dashboard');
    }
  }, [isAuthenticated, router]);

  const onSubmit = (data: LoginFormData) => {
    login(data);
  };

  // Extract error message
  const errorMessage = error
    ? (error as any)?.response?.data?.message || 'Login failed. Please try again.'
    : null;

  return (
    <AuthForm
      title="Welcome Back"
      subtitle="Sign in to continue your emotional journey"
      onSubmit={handleSubmit(onSubmit)}
    >
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

      {/* Email field */}
      <FormField label="Email" error={errors.email?.message}>
        <FormInput
          type="email"
          placeholder="you@example.com"
          error={!!errors.email}
          {...register('email')}
        />
      </FormField>

      {/* Password field */}
      <FormField label="Password" error={errors.password?.message}>
        <PasswordInput
          placeholder="Enter your password"
          error={!!errors.password}
          value={passwordValue}
          {...register('password')}
        />
      </FormField>

      {/* Forgot password link */}
      <div className="flex justify-end">
        <Link
          href="/forgot-password"
          className="text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 transition-colors"
        >
          Forgot password?
        </Link>
      </div>

      {/* Submit button */}
      <LoadingButton type="submit" loading={isPending}>
        Sign In
      </LoadingButton>

      {/* Register link */}
      <div className="text-center">
        <p className="text-sm text-gray-600 dark:text-gray-400">
          Don't have an account?{' '}
          <Link
            href="/register"
            className="font-medium text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 transition-colors"
          >
            Create one now
          </Link>
        </p>
      </div>
    </AuthForm>
  );
}
