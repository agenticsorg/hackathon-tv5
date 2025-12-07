/**
 * Authentication Middleware
 * Validates JWT tokens and extracts user context
 */

import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { config } from '../config.js';
import { UserContext, MCPErrorCode } from '../types/index.js';

export interface AuthenticatedRequest extends Request {
  userContext?: UserContext;
}

export async function authenticateRequest(
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): Promise<void> {
  const authHeader = req.headers.authorization;

  // Allow unauthenticated requests for public endpoints
  if (!authHeader) {
    req.userContext = {
      tier: 'free',
    };
    return next();
  }

  try {
    const token = authHeader.replace('Bearer ', '');
    const decoded = jwt.verify(token, config.auth.jwtSecret) as any;

    req.userContext = {
      userId: decoded.sub || decoded.userId,
      role: decoded.role || 'user',
      scopes: decoded.scopes || [],
      tier: decoded.tier || 'free',
    };

    next();
  } catch (error) {
    console.error('[Auth] Token verification failed:', error);
    res.status(401).json({
      error: {
        code: MCPErrorCode.AUTHORIZATION_ERROR,
        message: 'Invalid or expired token',
      },
    });
  }
}

export function requireAuth(
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): void {
  if (!req.userContext?.userId) {
    res.status(401).json({
      error: {
        code: MCPErrorCode.AUTHORIZATION_ERROR,
        message: 'Authentication required',
      },
    });
    return;
  }
  next();
}

export function requireScope(scope: string) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.userContext?.scopes?.includes(scope)) {
      res.status(403).json({
        error: {
          code: MCPErrorCode.AUTHORIZATION_ERROR,
          message: `Insufficient permissions. ${scope} scope required`,
        },
      });
      return;
    }
    next();
  };
}
