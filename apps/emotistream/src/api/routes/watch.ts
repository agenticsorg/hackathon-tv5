/**
 * Watch Tracking API Routes
 *
 * Routes for managing watch sessions.
 */

import { Router, Request, Response } from 'express';
import { z } from 'zod';
import { WatchTracker } from '../../services/watch-tracker.js';
import { apiResponse } from '../middleware/response.js';
import { ValidationError } from '../../utils/errors.js';

const router = Router();
const watchTracker = new WatchTracker();

// Validation schemas
const startSessionSchema = z.object({
  userId: z.string().min(1),
  contentId: z.string().min(1),
  contentTitle: z.string().min(1),
});

const sessionActionSchema = z.object({
  sessionId: z.string().min(1),
});

const endSessionSchema = z.object({
  sessionId: z.string().min(1),
  completed: z.boolean(),
});

/**
 * POST /api/v1/watch/start
 * Start a new watch session
 */
router.post('/start', async (req: Request, res: Response) => {
  try {
    const { userId, contentId, contentTitle } = startSessionSchema.parse(req.body);

    const session = watchTracker.startSession(userId, contentId, contentTitle);

    res.status(201).json(apiResponse({
      session: {
        sessionId: session.sessionId,
        userId: session.userId,
        contentId: session.contentId,
        contentTitle: session.contentTitle,
        startTime: session.startTime,
        status: 'active',
      },
    }));
  } catch (error) {
    if (error instanceof z.ZodError) {
      throw new ValidationError('Invalid request data', error.errors);
    }
    throw error;
  }
});

/**
 * POST /api/v1/watch/pause
 * Pause a watch session
 */
router.post('/pause', async (req: Request, res: Response) => {
  try {
    const { sessionId } = sessionActionSchema.parse(req.body);

    const session = watchTracker.pauseSession(sessionId);

    res.json(apiResponse({
      session: {
        sessionId: session.sessionId,
        status: 'paused',
        duration: session.duration,
        pauseCount: session.pauseCount,
      },
    }));
  } catch (error) {
    if (error instanceof z.ZodError) {
      throw new ValidationError('Invalid request data', error.errors);
    }
    throw error;
  }
});

/**
 * POST /api/v1/watch/resume
 * Resume a paused watch session
 */
router.post('/resume', async (req: Request, res: Response) => {
  try {
    const { sessionId } = sessionActionSchema.parse(req.body);

    const session = watchTracker.resumeSession(sessionId);

    res.json(apiResponse({
      session: {
        sessionId: session.sessionId,
        status: 'active',
      },
    }));
  } catch (error) {
    if (error instanceof z.ZodError) {
      throw new ValidationError('Invalid request data', error.errors);
    }
    throw error;
  }
});

/**
 * POST /api/v1/watch/end
 * End a watch session
 */
router.post('/end', async (req: Request, res: Response) => {
  try {
    const { sessionId, completed } = endSessionSchema.parse(req.body);

    const session = watchTracker.endSession(sessionId, completed);

    res.json(apiResponse({
      session: {
        sessionId: session.sessionId,
        status: 'ended',
        duration: session.duration,
        completed: session.completed,
        startTime: session.startTime,
        endTime: session.endTime,
      },
    }));
  } catch (error) {
    if (error instanceof z.ZodError) {
      throw new ValidationError('Invalid request data', error.errors);
    }
    throw error;
  }
});

/**
 * GET /api/v1/watch/:sessionId
 * Get watch session details
 */
router.get('/:sessionId', async (req: Request, res: Response) => {
  try {
    const { sessionId } = req.params;

    const session = watchTracker.getSession(sessionId);
    const elapsedTime = watchTracker.getElapsedTime(sessionId);

    res.json(apiResponse({
      session: {
        sessionId: session.sessionId,
        userId: session.userId,
        contentId: session.contentId,
        contentTitle: session.contentTitle,
        startTime: session.startTime,
        endTime: session.endTime,
        duration: session.duration,
        elapsedTime,
        completed: session.completed,
        paused: session.paused,
        pauseCount: session.pauseCount,
        status: session.endTime ? 'ended' : session.paused ? 'paused' : 'active',
      },
    }));
  } catch (error) {
    throw error;
  }
});

/**
 * GET /api/v1/watch/user/:userId
 * Get active watch sessions for user
 */
router.get('/user/:userId', async (req: Request, res: Response) => {
  try {
    const { userId } = req.params;

    const sessions = watchTracker.getUserSessions(userId);

    res.json(apiResponse({
      sessions: sessions.map(s => ({
        sessionId: s.sessionId,
        contentId: s.contentId,
        contentTitle: s.contentTitle,
        startTime: s.startTime,
        duration: s.duration,
        paused: s.paused,
      })),
    }));
  } catch (error) {
    throw error;
  }
});

export default router;
