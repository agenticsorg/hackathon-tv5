'use client';

import { motion } from 'framer-motion';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Activity, Cpu, Database, Globe, Zap, Users } from 'lucide-react';
import { useEffect, useState } from 'react';

interface Metric {
  label: string;
  value: string;
  change: string;
  icon: typeof Activity;
  color: string;
  trend: 'up' | 'down' | 'stable';
}

export function AIMetrics() {
  const [metrics, setMetrics] = useState<Metric[]>([
    {
      label: 'Connected TVs',
      value: '40,125,483',
      change: '+0.3%',
      icon: Users,
      color: 'text-purple-400',
      trend: 'up',
    },
    {
      label: 'Sync Requests/sec',
      value: '847,321',
      change: 'Live',
      icon: Activity,
      color: 'text-blue-400',
      trend: 'stable',
    },
    {
      label: 'Avg Query Latency',
      value: '12.4ms',
      change: '-2.1ms',
      icon: Zap,
      color: 'text-yellow-400',
      trend: 'down',
    },
    {
      label: 'Vector Database',
      value: '150.2M',
      change: '+1.2M',
      icon: Database,
      color: 'text-green-400',
      trend: 'up',
    },
    {
      label: 'Active Servers',
      value: '98',
      change: '99.99% uptime',
      icon: Globe,
      color: 'text-cyan-400',
      trend: 'stable',
    },
    {
      label: 'SIMD Acceleration',
      value: '13-41x',
      change: 'vs pgvector',
      icon: Cpu,
      color: 'text-pink-400',
      trend: 'stable',
    },
  ]);

  // Simulate real-time updates
  useEffect(() => {
    const interval = setInterval(() => {
      setMetrics((prev) =>
        prev.map((metric, i) => {
          if (metric.label === 'Sync Requests/sec') {
            const variance = Math.random() * 100000 - 50000;
            const newValue = Math.max(700000, Math.min(1200000, 847321 + variance));
            return {
              ...metric,
              value: Math.floor(newValue).toLocaleString(),
            };
          }
          if (metric.label === 'Avg Query Latency') {
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
    <div className="space-y-6">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="text-center space-y-2"
      >
        <h2 className="text-3xl font-bold bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
          Real-Time System Metrics
        </h2>
        <p className="text-gray-400">
          Live performance data from the distributed AI infrastructure
        </p>
      </motion.div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {metrics.map((metric, index) => {
          const Icon = metric.icon;
          return (
            <motion.div
              key={metric.label}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.3, delay: index * 0.1 }}
            >
              <Card className="border-gray-700 hover:border-gray-600 transition-all duration-300 hover:shadow-lg hover:shadow-purple-500/20">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium text-gray-400">
                    {metric.label}
                  </CardTitle>
                  <Icon className={`w-4 h-4 ${metric.color}`} />
                </CardHeader>
                <CardContent>
                  <div className="flex items-baseline justify-between">
                    <motion.div
                      key={metric.value}
                      initial={{ opacity: 0, y: -10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ duration: 0.3 }}
                      className={`text-2xl font-bold ${metric.color}`}
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

      {/* System Status Indicator */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.6 }}
      >
        <Card className="border-green-800/50 bg-green-900/10">
          <CardContent className="flex items-center justify-center gap-3 py-4">
            <motion.div
              animate={{ scale: [1, 1.2, 1] }}
              transition={{ duration: 2, repeat: Infinity }}
              className="w-3 h-3 rounded-full bg-green-500"
            />
            <span className="text-green-400 font-semibold">
              All Systems Operational
            </span>
            <Badge variant="success" className="ml-2">
              99.99% Uptime
            </Badge>
          </CardContent>
        </Card>
      </motion.div>
    </div>
  );
}
