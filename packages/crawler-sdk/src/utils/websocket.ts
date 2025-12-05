/**
 * WebSocket client for real-time crawl updates
 */

import WebSocket from 'ws';
import { WebSocketMessage, StreamOptions, CrawlerConfig } from '../types';
import { CrawlerError } from './errors';

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private debug: boolean;

  constructor(private config: CrawlerConfig) {
    this.debug = config.debug || false;
  }

  /**
   * Connect to WebSocket and stream crawl results
   */
  async connect(
    crawlId: string,
    options: StreamOptions
  ): Promise<void> {
    const baseUrl = this.config.baseUrl || 'http://localhost:3000';
    const wsUrl = baseUrl.replace(/^http/, 'ws') + `/api/crawl/${crawlId}/stream`;

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(wsUrl, {
          headers: this.config.apiKey
            ? { Authorization: `Bearer ${this.config.apiKey}` }
            : {},
        });

        this.ws.on('open', () => {
          if (this.debug) {
            console.log('[WebSocket] Connected to', wsUrl);
          }
          this.reconnectAttempts = 0;
          resolve();
        });

        this.ws.on('message', (data: WebSocket.Data) => {
          try {
            const message: WebSocketMessage = JSON.parse(data.toString());
            this.handleMessage(message, options);
          } catch (error) {
            if (this.debug) {
              console.error('[WebSocket] Failed to parse message:', error);
            }
          }
        });

        this.ws.on('error', (error) => {
          if (this.debug) {
            console.error('[WebSocket] Error:', error);
          }
          options.onError?.(new CrawlerError(`WebSocket error: ${error.message}`));
        });

        this.ws.on('close', () => {
          if (this.debug) {
            console.log('[WebSocket] Connection closed');
          }
          this.handleReconnect(crawlId, options);
        });
      } catch (error) {
        reject(
          new CrawlerError(
            `Failed to connect to WebSocket: ${error instanceof Error ? error.message : 'Unknown error'}`
          )
        );
      }
    });
  }

  /**
   * Handle incoming WebSocket messages
   */
  private handleMessage(
    message: WebSocketMessage,
    options: StreamOptions
  ): void {
    switch (message.type) {
      case 'status':
        options.onStatus?.(message.data);
        break;
      case 'result':
        options.onResult?.(message.data);
        break;
      case 'error':
        options.onError?.(new CrawlerError(message.data.error));
        break;
      case 'complete':
        options.onComplete?.();
        this.close();
        break;
    }
  }

  /**
   * Attempt to reconnect on connection loss
   */
  private async handleReconnect(
    crawlId: string,
    options: StreamOptions
  ): Promise<void> {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

      if (this.debug) {
        console.log(
          `[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`
        );
      }

      await this.sleep(delay);
      await this.connect(crawlId, options);
    } else {
      options.onError?.(
        new CrawlerError('Max reconnection attempts reached')
      );
    }
  }

  /**
   * Close the WebSocket connection
   */
  close(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Check if WebSocket is connected
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Sleep helper
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
