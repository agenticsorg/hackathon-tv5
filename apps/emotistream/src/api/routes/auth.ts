import { Router, Request, Response } from 'express';
import { JWTService } from '../../auth/jwt-service';
import { PasswordService } from '../../auth/password-service';
import { UserStore, User } from '../../persistence/user-store';

export interface RegisterRequest {
  email: string;
  password: string;
  dateOfBirth?: string;
  displayName?: string;
  name?: string; // Alternative to displayName for frontend compatibility
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RefreshRequest {
  refreshToken: string;
}

/**
 * Create authentication router
 */
export function createAuthRouter(
  jwtService: JWTService,
  passwordService: PasswordService,
  userStore: UserStore
): Router {
  const router = Router();

  /**
   * POST /api/v1/auth/register
   * Register a new user
   */
  router.post('/register', async (req: Request, res: Response): Promise<void> => {
    try {
      const { email, password, dateOfBirth, displayName, name } = req.body as RegisterRequest;

      // Use name as fallback for displayName (frontend compatibility)
      const finalDisplayName = displayName || name;

      // Validate input - only email, password, and displayName/name are required
      if (!email || !password || !finalDisplayName) {
        res.status(400).json({
          success: false,
          data: null,
          error: {
            code: 'E003',
            message: 'Missing required fields',
            details: {
              required: ['email', 'password', 'name or displayName']
            }
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Validate email format
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(email)) {
        res.status(400).json({
          success: false,
          data: null,
          error: {
            code: 'E003',
            message: 'Invalid email format',
            details: {}
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Validate password strength
      const passwordErrors = passwordService.validate(password);
      if (passwordErrors.length > 0) {
        res.status(400).json({
          success: false,
          data: null,
          error: {
            code: 'E003',
            message: 'Password validation failed',
            details: {
              errors: passwordErrors
            }
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Check if email already exists
      if (userStore.getByEmail(email)) {
        res.status(400).json({
          success: false,
          data: null,
          error: {
            code: 'E003',
            message: 'Email already registered',
            details: {}
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Hash password
      const hashedPassword = await passwordService.hash(password);

      // Create user
      const userId = userStore.generateUserId();
      const now = Date.now();
      const user: User = {
        id: userId,
        email,
        password: hashedPassword,
        displayName: finalDisplayName,
        dateOfBirth: dateOfBirth || '', // Optional field
        createdAt: now,
        lastActive: now
      };

      userStore.create(user);

      // Generate tokens
      const token = jwtService.generateAccessToken(userId);
      const refreshToken = jwtService.generateRefreshToken(userId);
      const expiresAt = jwtService.getExpirationTime(token);

      res.status(201).json({
        success: true,
        data: {
          userId,
          email,
          displayName: finalDisplayName,
          token,
          refreshToken,
          expiresAt: expiresAt.toISOString(),
          createdAt: new Date(now).toISOString()
        },
        error: null,
        timestamp: new Date().toISOString()
      });
    } catch (error) {
      console.error('Register error:', error);
      res.status(500).json({
        success: false,
        data: null,
        error: {
          code: 'E010',
          message: 'Internal server error',
          details: {}
        },
        timestamp: new Date().toISOString()
      });
    }
  });

  /**
   * POST /api/v1/auth/login
   * Login with email and password
   */
  router.post('/login', async (req: Request, res: Response): Promise<void> => {
    try {
      const { email, password } = req.body as LoginRequest;

      // Validate input
      if (!email || !password) {
        res.status(400).json({
          success: false,
          data: null,
          error: {
            code: 'E003',
            message: 'Missing email or password',
            details: {}
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Get user by email
      const user = userStore.getByEmail(email);
      if (!user) {
        res.status(401).json({
          success: false,
          data: null,
          error: {
            code: 'E007',
            message: 'Invalid email or password',
            details: {}
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Verify password
      const isValid = await passwordService.verify(password, user.password);
      if (!isValid) {
        res.status(401).json({
          success: false,
          data: null,
          error: {
            code: 'E007',
            message: 'Invalid email or password',
            details: {}
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Update last active
      userStore.updateLastActive(user.id);

      // Generate tokens
      const token = jwtService.generateAccessToken(user.id);
      const refreshToken = jwtService.generateRefreshToken(user.id);
      const expiresAt = jwtService.getExpirationTime(token);

      res.status(200).json({
        success: true,
        data: {
          userId: user.id,
          email: user.email,
          displayName: user.displayName,
          token,
          refreshToken,
          expiresAt: expiresAt.toISOString(),
          createdAt: new Date(user.createdAt).toISOString()
        },
        error: null,
        timestamp: new Date().toISOString()
      });
    } catch (error) {
      console.error('Login error:', error);
      res.status(500).json({
        success: false,
        data: null,
        error: {
          code: 'E010',
          message: 'Internal server error',
          details: {}
        },
        timestamp: new Date().toISOString()
      });
    }
  });

  /**
   * POST /api/v1/auth/refresh
   * Refresh access token using refresh token
   */
  router.post('/refresh', async (req: Request, res: Response): Promise<void> => {
    try {
      const { refreshToken } = req.body as RefreshRequest;

      if (!refreshToken) {
        res.status(400).json({
          success: false,
          data: null,
          error: {
            code: 'E003',
            message: 'Missing refresh token',
            details: {}
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Verify refresh token
      let payload;
      try {
        payload = jwtService.verifyRefreshToken(refreshToken);
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Invalid refresh token';
        res.status(401).json({
          success: false,
          data: null,
          error: {
            code: 'E007',
            message: `Invalid refresh token: ${message}`,
            details: {}
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Verify user exists
      const user = userStore.getById(payload.userId);
      if (!user) {
        res.status(401).json({
          success: false,
          data: null,
          error: {
            code: 'E007',
            message: 'User not found',
            details: {}
          },
          timestamp: new Date().toISOString()
        });
        return;
      }

      // Generate new access token
      const token = jwtService.generateAccessToken(user.id);
      const expiresAt = jwtService.getExpirationTime(token);

      res.status(200).json({
        success: true,
        data: {
          token,
          expiresAt: expiresAt.toISOString()
        },
        error: null,
        timestamp: new Date().toISOString()
      });
    } catch (error) {
      console.error('Refresh error:', error);
      res.status(500).json({
        success: false,
        data: null,
        error: {
          code: 'E010',
          message: 'Internal server error',
          details: {}
        },
        timestamp: new Date().toISOString()
      });
    }
  });

  return router;
}
