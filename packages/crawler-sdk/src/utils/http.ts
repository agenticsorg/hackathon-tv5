/**
 * HTTP client with retry logic and error handling
 */

import axios, { AxiosInstance, AxiosRequestConfig, AxiosError } from 'axios';
import { CrawlerConfig } from '../types';
import {
  CrawlerError,
  NetworkError,
  TimeoutError,
  createErrorFromResponse,
} from './errors';

export class HttpClient {
  private client: AxiosInstance;
  private retryAttempts: number;
  private retryDelay: number;
  private debug: boolean;

  constructor(config: CrawlerConfig) {
    this.retryAttempts = config.retryAttempts || 3;
    this.retryDelay = config.retryDelay || 1000;
    this.debug = config.debug || false;

    this.client = axios.create({
      baseURL: config.baseUrl || 'http://localhost:3000/api',
      timeout: config.timeout || 30000,
      headers: {
        'Content-Type': 'application/json',
        ...(config.apiKey && { Authorization: `Bearer ${config.apiKey}` }),
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors(): void {
    // Request interceptor
    this.client.interceptors.request.use(
      (config) => {
        if (this.debug) {
          console.log('[HTTP Request]', config.method?.toUpperCase(), config.url);
        }
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor
    this.client.interceptors.response.use(
      (response) => {
        if (this.debug) {
          console.log('[HTTP Response]', response.status, response.config.url);
        }
        return response;
      },
      (error) => {
        if (this.debug) {
          console.error('[HTTP Error]', error.message);
        }
        return Promise.reject(error);
      }
    );
  }

  /**
   * Exponential backoff delay calculation
   */
  private calculateDelay(attempt: number): number {
    return this.retryDelay * Math.pow(2, attempt);
  }

  /**
   * Determines if an error is retryable
   */
  private isRetryableError(error: AxiosError): boolean {
    if (!error.response) {
      // Network errors are retryable
      return true;
    }

    const status = error.response.status;
    // Retry on server errors (5xx) and rate limiting (429)
    return status >= 500 || status === 429;
  }

  /**
   * Makes an HTTP request with retry logic
   */
  async request<T>(
    config: AxiosRequestConfig,
    attempt = 0
  ): Promise<T> {
    try {
      const response = await this.client.request<T>(config);
      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        // Handle timeout
        if (error.code === 'ECONNABORTED') {
          throw new TimeoutError(
            `Request timed out after ${config.timeout || 30000}ms`
          );
        }

        // Retry logic
        if (this.isRetryableError(error) && attempt < this.retryAttempts) {
          const delay = this.calculateDelay(attempt);
          if (this.debug) {
            console.log(
              `[HTTP Retry] Attempt ${attempt + 1}/${this.retryAttempts} after ${delay}ms`
            );
          }
          await this.sleep(delay);
          return this.request<T>(config, attempt + 1);
        }

        // Create appropriate error
        if (error.response) {
          const message =
            error.response.data?.error ||
            error.response.data?.message ||
            error.message;
          throw createErrorFromResponse(
            error.response.status,
            message,
            error.response.data
          );
        } else {
          throw new NetworkError(
            `Network error: ${error.message}`,
            error
          );
        }
      }

      // Unknown error
      throw new CrawlerError(
        error instanceof Error ? error.message : 'Unknown error occurred'
      );
    }
  }

  /**
   * GET request
   */
  async get<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    return this.request<T>({ ...config, method: 'GET', url });
  }

  /**
   * POST request
   */
  async post<T>(
    url: string,
    data?: any,
    config?: AxiosRequestConfig
  ): Promise<T> {
    return this.request<T>({ ...config, method: 'POST', url, data });
  }

  /**
   * PUT request
   */
  async put<T>(
    url: string,
    data?: any,
    config?: AxiosRequestConfig
  ): Promise<T> {
    return this.request<T>({ ...config, method: 'PUT', url, data });
  }

  /**
   * DELETE request
   */
  async delete<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    return this.request<T>({ ...config, method: 'DELETE', url });
  }

  /**
   * Sleep helper for retry delays
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Update authorization header
   */
  setApiKey(apiKey: string): void {
    this.client.defaults.headers.common['Authorization'] = `Bearer ${apiKey}`;
  }

  /**
   * Get the underlying Axios instance
   */
  getAxiosInstance(): AxiosInstance {
    return this.client;
  }
}
