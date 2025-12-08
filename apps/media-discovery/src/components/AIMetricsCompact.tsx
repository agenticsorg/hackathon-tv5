'use client';

import { motion } from 'framer-motion';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Activity, Zap, Users } from 'lucide-react';
import { useEffect, useState } from 'react';

interface Metric {
  label: string;
  value: string;
  change: string;
  icon: typeof Activity;
  color: string;
  trend: 'up' | 'down' | 'stable';
}

export function AIMetricsCompact() {
  const [metrics, setMetrics] = useState<Metric[]>([
    {
      label: 'Connected TVs',
      value: '40.1M',
      change: '+0.3%',
      icon: Users,
      color: 'text-purple-400',
      trend: 'up',
    },
    {
      label: 'Sync Requests/sec',
      value: '847K',
      change: 'Live',
      icon: Activity,
      color: 'text-blue-400',
      trend: 'stable',
    },
    {
      label: 'Query Latency',
      value: '12.4ms',
      change: '-2.1ms',
      icon: Zap,
      color: 'text-yellow-400',
      trend: 'down',
    },
  ]);

  // Simulate real-time updates
  useEffect(() => {
    const interval = setInterval(() => {
      setMetrics((prev) =>
        prev.map((metric) => {
          if (metric.label === 'Sync Requests/sec') {
            const variance = Math.random() * 100 - 50;
            const newValue = Math.max(700, Math.min(1200, 847 + variance));
            return {
              ...metric,
              value: `${Math.floor(newValue)}K`,
            };
          }
          if (metric.label === 'Query Latency') {
            const variance = Math.random() * 4 - 2;
            const newValue = Math.max(10, Math.min(18, 12.4 + variance));
            return {
              ...metric,
              value: `${newValue.toFixed(1)}ms`,
            };
          }
          return metric;
        })
      );
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="space-y-4">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="space-y-2"
      >
        <div className="flex items-center gap-2">
          <motion.div
            animate={{ scale: [1, 1.2, 1] }}
            transition={{ duration: 2, repeat: Infinity }}
            className="w-2 h-2 rounded-full bg-green-500"
          />
          <h3 className="text-lg font-bold bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
            Live System Status
          </h3>
        </div>
        <p className="text-xs text-gray-400">
          Real-time metrics from 40M+ TVs
        </p>
      </motion.div>

      <div className="space-y-3">
        {metrics.map((metric, index) => {
          const Icon = metric.icon;
          return (
            <motion.div
              key={metric.label}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.3, delay: index * 0.1 }}
            >
              <Card className="border-gray-700 hover:border-gray-600 transition-all duration-300 hover:shadow-lg hover:shadow-purple-500/20">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2 px-4 pt-4">
                  <CardTitle className="text-xs font-medium text-gray-400">
                    {metric.label}
                  </CardTitle>
                  <Icon className={`w-3.5 h-3.5 ${metric.color}`} />
                </CardHeader>
                <CardContent className="px-4 pb-4">
                  <div className="flex items-baseline justify-between">
                    <motion.div
                      key={metric.value}
                      initial={{ opacity: 0, y: -10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ duration: 0.3 }}
                      className={`text-xl font-bold ${metric.color}`}
                    >
                      {metric.value}
                    </motion.div>
                    <Badge
                      variant={metric.trend === 'up' ? 'success' : metric.trend === 'down' ? 'warning' : 'secondary'}
                      className="text-xs"
                    >
                      {metric.change}
                    </Badge>
                  </div>
                </CardContent>
              </Card>
            </motion.div>
          );
        })}
      </div>

      {/* Compact Status Indicator */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.3 }}
      >
        <Card className="border-green-800/50 bg-green-900/10">
          <CardContent className="flex items-center justify-center gap-2 py-3 px-4">
            <span className="text-xs text-green-400 font-semibold">
              99.99% Uptime
            </span>
            <Badge variant="success" className="text-xs">
              All Systems Up
            </Badge>
          </CardContent>
        </Card>
      </motion.div>
    </div>
  );
}
