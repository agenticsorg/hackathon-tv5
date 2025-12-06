import jwt from 'jsonwebtoken';

export interface TokenPayload {
  userId: string;
  type?: 'refresh';
}

/**
 * JWT Service for token generation and verification
 */
export class JWTService {
  private secret: string;

  constructor(secret?: string) {
    this.secret = secret || process.env.JWT_SECRET || 'emotistream-dev-secret-change-in-production';
  }

  /**
   * Generate access token (24h expiry)
   */
  generateAccessToken(userId: string): string {
    return jwt.sign({ userId }, this.secret, { expiresIn: '24h' });
  }

  /**
   * Generate refresh token (7d expiry)
   */
  generateRefreshToken(userId: string): string {
    return jwt.sign({ userId, type: 'refresh' }, this.secret, { expiresIn: '7d' });
  }

  /**
   * Verify and decode token
   * @throws Error if token is invalid or expired
   */
  verify(token: string): TokenPayload {
    try {
      const decoded = jwt.verify(token, this.secret) as TokenPayload;
      return decoded;
    } catch (error) {
      if (error instanceof jwt.TokenExpiredError) {
        throw new Error('Token expired');
      }
      if (error instanceof jwt.JsonWebTokenError) {
        throw new Error('Invalid token');
      }
      throw error;
    }
  }

  /**
   * Verify refresh token
   * @throws Error if token is not a refresh token or is invalid
   */
  verifyRefreshToken(token: string): TokenPayload {
    const decoded = this.verify(token);
    if (decoded.type !== 'refresh') {
      throw new Error('Not a refresh token');
    }
    return decoded;
  }

  /**
   * Get token expiration timestamp
   */
  getExpirationTime(token: string): Date {
    const decoded = jwt.decode(token) as { exp?: number };
    if (!decoded || !decoded.exp) {
      throw new Error('Invalid token');
    }
    return new Date(decoded.exp * 1000);
  }
}
