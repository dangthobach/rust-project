/**
 * Performance Monitor Component
 * Displays real-time performance metrics
 */

import { Component, createSignal, onCleanup, onMount } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Badge } from '~/components/ui';

interface PerformanceMetrics {
  apiResponseTime: number;
  cacheHitRate: number;
  memoryUsage: number;
  activeConnections: number;
  requestsPerMinute: number;
}

const PerformanceMonitor: Component = () => {
  const [metrics, setMetrics] = createSignal<PerformanceMetrics>({
    apiResponseTime: 0,
    cacheHitRate: 0,
    memoryUsage: 0,
    activeConnections: 0,
    requestsPerMinute: 0,
  });

  const [isMonitoring, setIsMonitoring] = createSignal(false);

  onMount(() => {
    setIsMonitoring(true);
    
    // Simulate performance monitoring (in production, fetch from actual metrics API)
    const interval = setInterval(() => {
      // Mock data - replace with real API calls
      setMetrics({
        apiResponseTime: Math.random() * 200 + 50, // 50-250ms
        cacheHitRate: Math.random() * 30 + 70, // 70-100%
        memoryUsage: Math.random() * 20 + 40, // 40-60%
        activeConnections: Math.floor(Math.random() * 50 + 10), // 10-60 connections
        requestsPerMinute: Math.floor(Math.random() * 100 + 50), // 50-150 req/min
      });
    }, 2000);

    onCleanup(() => {
      clearInterval(interval);
      setIsMonitoring(false);
    });
  });

  const getPerformanceStatus = (value: number, metric: string): { color: string; label: string } => {
    if (metric === 'responseTime') {
      if (value < 100) return { color: 'bg-green-500', label: 'Excellent' };
      if (value < 200) return { color: 'bg-yellow-500', label: 'Good' };
      return { color: 'bg-red-500', label: 'Slow' };
    }
    
    if (metric === 'cacheHitRate') {
      if (value > 85) return { color: 'bg-green-500', label: 'Excellent' };
      if (value > 70) return { color: 'bg-yellow-500', label: 'Good' };
      return { color: 'bg-red-500', label: 'Poor' };
    }
    
    if (metric === 'memoryUsage') {
      if (value < 60) return { color: 'bg-green-500', label: 'Normal' };
      if (value < 80) return { color: 'bg-yellow-500', label: 'High' };
      return { color: 'bg-red-500', label: 'Critical' };
    }
    
    return { color: 'bg-gray-500', label: 'Unknown' };
  };

  return (
    <Card class="border-5">
      <CardHeader>
        <div class="flex items-center justify-between">
          <CardTitle class="flex items-center gap-2">
            <span class="text-2xl">⚡</span>
            Performance Monitor
          </CardTitle>
          <Badge 
            variant={isMonitoring() ? 'success' : 'error'}
            class="border-3 flex items-center gap-2"
          >
            <span class={`w-2 h-2 rounded-full ${isMonitoring() ? 'bg-green-500 animate-pulse' : 'bg-red-500'}`} />
            {isMonitoring() ? 'Live' : 'Stopped'}
          </Badge>
        </div>
      </CardHeader>
      <CardContent class="space-y-4">
        {/* API Response Time */}
        <div class="p-4 border-3 border-black bg-white">
          <div class="flex items-center justify-between mb-2">
            <span class="font-bold text-sm uppercase">API Response Time</span>
            <Badge 
              class={`${getPerformanceStatus(metrics().apiResponseTime, 'responseTime').color} border-3`}
            >
              {getPerformanceStatus(metrics().apiResponseTime, 'responseTime').label}
            </Badge>
          </div>
          <div class="text-2xl font-heading font-black">
            {metrics().apiResponseTime.toFixed(0)}ms
          </div>
          <div class="w-full bg-neutral-concrete h-3 border-3 border-black mt-2">
            <div
              class={`h-full ${getPerformanceStatus(metrics().apiResponseTime, 'responseTime').color} transition-all duration-300`}
              style={{ width: `${Math.min((metrics().apiResponseTime / 300) * 100, 100)}%` }}
            />
          </div>
        </div>

        {/* Cache Hit Rate */}
        <div class="p-4 border-3 border-black bg-white">
          <div class="flex items-center justify-between mb-2">
            <span class="font-bold text-sm uppercase">Cache Hit Rate</span>
            <Badge 
              class={`${getPerformanceStatus(metrics().cacheHitRate, 'cacheHitRate').color} border-3`}
            >
              {getPerformanceStatus(metrics().cacheHitRate, 'cacheHitRate').label}
            </Badge>
          </div>
          <div class="text-2xl font-heading font-black">
            {metrics().cacheHitRate.toFixed(1)}%
          </div>
          <div class="w-full bg-neutral-concrete h-3 border-3 border-black mt-2">
            <div
              class={`h-full ${getPerformanceStatus(metrics().cacheHitRate, 'cacheHitRate').color} transition-all duration-300`}
              style={{ width: `${metrics().cacheHitRate}%` }}
            />
          </div>
        </div>

        {/* Memory Usage */}
        <div class="p-4 border-3 border-black bg-white">
          <div class="flex items-center justify-between mb-2">
            <span class="font-bold text-sm uppercase">Memory Usage</span>
            <Badge 
              class={`${getPerformanceStatus(metrics().memoryUsage, 'memoryUsage').color} border-3`}
            >
              {getPerformanceStatus(metrics().memoryUsage, 'memoryUsage').label}
            </Badge>
          </div>
          <div class="text-2xl font-heading font-black">
            {metrics().memoryUsage.toFixed(1)}%
          </div>
          <div class="w-full bg-neutral-concrete h-3 border-3 border-black mt-2">
            <div
              class={`h-full ${getPerformanceStatus(metrics().memoryUsage, 'memoryUsage').color} transition-all duration-300`}
              style={{ width: `${metrics().memoryUsage}%` }}
            />
          </div>
        </div>

        {/* Quick Stats */}
        <div class="grid grid-cols-2 gap-3">
          <div class="p-3 border-3 border-black bg-primary text-center">
            <div class="text-xl font-heading font-black">{metrics().activeConnections}</div>
            <div class="text-xs font-bold uppercase">Active Connections</div>
          </div>
          <div class="p-3 border-3 border-black bg-secondary text-center">
            <div class="text-xl font-heading font-black">{metrics().requestsPerMinute}</div>
            <div class="text-xs font-bold uppercase">Requests/Min</div>
          </div>
        </div>

        {/* Performance Tips */}
        <div class="p-3 border-3 border-black bg-accent-yellow">
          <div class="font-bold text-sm mb-2">💡 Performance Tips:</div>
          <ul class="text-xs space-y-1 list-disc list-inside">
            <li>Enable caching for frequently accessed data</li>
            <li>Use pagination for large datasets</li>
            <li>Optimize database queries with indexes</li>
            <li>Implement lazy loading for images and components</li>
          </ul>
        </div>
      </CardContent>
    </Card>
  );
};

export default PerformanceMonitor;
