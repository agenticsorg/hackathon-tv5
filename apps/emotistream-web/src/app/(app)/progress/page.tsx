'use client';

import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { TrendingUp, Target, Activity, Award, AlertCircle, RefreshCw } from 'lucide-react';
import { useAuthStore } from '@/lib/stores/auth-store';
import { getProgress, getConvergence, getRewardTimeline, type ProgressData, type ConvergenceData, type RewardTimelinePoint } from '@/lib/api/progress';

export default function ProgressPage() {
  const { user } = useAuthStore();
  const [progress, setProgress] = useState<ProgressData | null>(null);
  const [convergence, setConvergence] = useState<ConvergenceData | null>(null);
  const [timeline, setTimeline] = useState<RewardTimelinePoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    if (!user?.id) {
      console.log('No user ID available');
      setLoading(false);
      return;
    }

    console.log('Fetching progress for user:', user.id);
    setLoading(true);
    setError(null);

    try {
      const [progressData, convergenceData, rewardData] = await Promise.all([
        getProgress(user.id),
        getConvergence(user.id),
        getRewardTimeline(user.id),
      ]);

      console.log('Progress data received:', progressData);
      setProgress(progressData);
      setConvergence(convergenceData);
      setTimeline(rewardData.timeline);
    } catch (err) {
      console.error('Failed to fetch progress:', err);
      setError('Failed to load progress data. Make sure you have some feedback history.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, [user?.id]);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-8">
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <h1 className="text-3xl font-bold text-gray-800 dark:text-white">
            Learning Progress
          </h1>
        </motion.div>

        <div className="p-6 bg-yellow-50 dark:bg-yellow-900/20 rounded-2xl border border-yellow-200 dark:border-yellow-800">
          <div className="flex items-start gap-3">
            <AlertCircle className="w-6 h-6 text-yellow-600 flex-shrink-0 mt-0.5" />
            <div>
              <h3 className="font-semibold text-yellow-800 dark:text-yellow-200">No Progress Data Yet</h3>
              <p className="text-yellow-700 dark:text-yellow-300 mt-1">
                Start watching content and providing feedback to see your learning progress here.
                Go to the Dashboard to analyze your emotions and get recommendations!
              </p>
              <button
                onClick={fetchData}
                className="mt-4 flex items-center gap-2 px-4 py-2 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700 transition"
              >
                <RefreshCw className="w-4 h-4" />
                Refresh
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  const metrics = [
    {
      label: 'Total Experiences',
      value: progress?.totalExperiences?.toString() || '0',
      icon: Activity,
      color: 'blue'
    },
    {
      label: 'Average Reward',
      value: progress?.averageReward || '0.00',
      icon: Award,
      color: 'green'
    },
    {
      label: 'Exploration Rate',
      value: progress?.explorationRate || '0%',
      icon: Target,
      color: 'purple'
    },
    {
      label: 'Convergence',
      value: `${convergence?.score || 0}%`,
      icon: TrendingUp,
      color: 'orange'
    },
  ];

  const convergencePercentage = convergence?.progressBar?.percentage || 0;

  return (
    <div className="space-y-8">
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="flex items-center justify-between"
      >
        <div>
          <h1 className="text-3xl font-bold text-gray-800 dark:text-white">
            Learning Progress
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            Track how well the system is learning your preferences.
          </p>
        </div>
        <button
          onClick={fetchData}
          className="flex items-center gap-2 px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition"
        >
          <RefreshCw className="w-4 h-4" />
          Refresh
        </button>
      </motion.div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {metrics.map((metric, index) => {
          const Icon = metric.icon;
          const colorClasses: Record<string, { bg: string; icon: string }> = {
            blue: { bg: 'bg-blue-100 dark:bg-blue-900/30', icon: 'text-blue-600' },
            green: { bg: 'bg-green-100 dark:bg-green-900/30', icon: 'text-green-600' },
            purple: { bg: 'bg-purple-100 dark:bg-purple-900/30', icon: 'text-purple-600' },
            orange: { bg: 'bg-orange-100 dark:bg-orange-900/30', icon: 'text-orange-600' },
          };
          const colors = colorClasses[metric.color] || colorClasses.blue;

          return (
            <motion.div
              key={metric.label}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.1 }}
              className="p-6 bg-white dark:bg-gray-800 rounded-2xl shadow-lg"
            >
              <div className={`w-12 h-12 rounded-xl ${colors.bg} flex items-center justify-center mb-4`}>
                <Icon className={`w-6 h-6 ${colors.icon}`} />
              </div>
              <p className="text-3xl font-bold text-gray-800 dark:text-white">
                {metric.value}
              </p>
              <p className="text-gray-600 dark:text-gray-400 text-sm">
                {metric.label}
              </p>
            </motion.div>
          );
        })}
      </div>

      {/* Convergence Indicator */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.4 }}
        className="p-6 bg-white dark:bg-gray-800 rounded-2xl shadow-lg"
      >
        <h2 className="text-xl font-semibold mb-4">Learning Confidence</h2>
        <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <motion.div
            className="h-full bg-gradient-to-r from-yellow-400 via-blue-500 to-green-500"
            initial={{ width: 0 }}
            animate={{ width: `${convergencePercentage}%` }}
            transition={{ duration: 1, delay: 0.5 }}
          />
        </div>
        <div className="flex justify-between mt-2 text-sm text-gray-500">
          <span>Still exploring</span>
          <span>Learning patterns</span>
          <span>Confident</span>
        </div>
        <p className="mt-4 text-gray-600 dark:text-gray-400">
          {convergence?.explanation || progress?.summary?.description || 'Keep providing feedback to improve recommendations!'}
        </p>
        {progress?.summary?.nextMilestone && (
          <p className="mt-2 text-sm text-blue-600 dark:text-blue-400">
            Next milestone: {progress.summary.nextMilestone.description}
          </p>
        )}
      </motion.div>

      {/* Reward Timeline */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.6 }}
        className="p-6 bg-white dark:bg-gray-800 rounded-2xl shadow-lg"
      >
        <h2 className="text-xl font-semibold mb-4">Reward Timeline</h2>
        {timeline.length > 0 ? (
          <div className="space-y-4">
            {/* Simple bar chart visualization */}
            <div className="flex items-end gap-1 h-32">
              {timeline.slice(-20).map((point, i) => {
                const normalizedHeight = Math.max(10, ((point.reward + 1) / 2) * 100);
                const isPositive = point.reward >= 0;
                return (
                  <div
                    key={i}
                    className="flex-1 group relative"
                  >
                    <div
                      className={`w-full rounded-t transition-all ${
                        isPositive
                          ? 'bg-green-500 hover:bg-green-400'
                          : 'bg-red-500 hover:bg-red-400'
                      }`}
                      style={{ height: `${normalizedHeight}%` }}
                    />
                    <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 opacity-0 group-hover:opacity-100 transition-opacity bg-gray-900 text-white text-xs px-2 py-1 rounded whitespace-nowrap z-10">
                      {point.contentTitle?.slice(0, 20)}...<br />
                      Reward: {point.reward.toFixed(3)}
                    </div>
                  </div>
                );
              })}
            </div>
            <p className="text-sm text-gray-500 text-center">
              Last {Math.min(20, timeline.length)} experiences
            </p>
          </div>
        ) : (
          <div className="h-64 flex items-center justify-center text-gray-400">
            <p>Complete some content experiences to see your reward timeline</p>
          </div>
        )}
      </motion.div>

      {/* Recent Experiences */}
      {timeline.length > 0 && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.8 }}
          className="p-6 bg-white dark:bg-gray-800 rounded-2xl shadow-lg"
        >
          <h2 className="text-xl font-semibold mb-4">Recent Experiences</h2>
          <div className="space-y-3">
            {timeline.slice(-5).reverse().map((exp, i) => (
              <div
                key={i}
                className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
              >
                <div>
                  <p className="font-medium text-gray-800 dark:text-white">
                    {exp.contentTitle || exp.contentId}
                  </p>
                  <p className="text-sm text-gray-500">
                    {new Date(exp.timestamp).toLocaleDateString()}
                  </p>
                </div>
                <div className="text-right">
                  <p className={`font-bold ${exp.reward >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                    {exp.reward >= 0 ? '+' : ''}{exp.reward.toFixed(3)}
                  </p>
                  <p className="text-sm text-gray-500">
                    {exp.starRating ? `${exp.starRating}★` : 'No rating'}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      )}
    </div>
  );
}
