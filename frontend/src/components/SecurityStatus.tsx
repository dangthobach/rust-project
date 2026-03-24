/**
 * Security Status Component
 * Displays security metrics and alerts
 */

import { Component, createSignal, For } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent, Badge, Button } from '~/components/ui';

interface SecurityAlert {
  id: string;
  level: 'critical' | 'warning' | 'info';
  title: string;
  description: string;
  timestamp: string;
}

interface SecurityMetrics {
  failedLoginAttempts: number;
  activeTokens: number;
  rateLimitViolations: number;
  securityScore: number;
}

const SecurityStatus: Component = () => {
  const [metrics] = createSignal<SecurityMetrics>({
    failedLoginAttempts: 3,
    activeTokens: 24,
    rateLimitViolations: 2,
    securityScore: 95,
  });

  const [alerts] = createSignal<SecurityAlert[]>([
    {
      id: '1',
      level: 'warning',
      title: 'Rate Limit Violations',
      description: '2 IP addresses have been rate limited in the last hour',
      timestamp: new Date().toISOString(),
    },
    {
      id: '2',
      level: 'info',
      title: 'Failed Login Attempts',
      description: '3 failed login attempts from different IPs',
      timestamp: new Date().toISOString(),
    },
  ]);

  const getSecurityLevel = (score: number): { color: string; label: string; emoji: string } => {
    if (score >= 90) return { color: 'bg-green-500', label: 'Excellent', emoji: '🛡️' };
    if (score >= 75) return { color: 'bg-yellow-500', label: 'Good', emoji: '⚠️' };
    if (score >= 50) return { color: 'bg-orange-500', label: 'Moderate', emoji: '⚡' };
    return { color: 'bg-red-500', label: 'Critical', emoji: '🚨' };
  };

  const getAlertBadge = (level: string) => {
    switch (level) {
      case 'critical':
        return { variant: 'error' as const, emoji: '🚨' };
      case 'warning':
        return { variant: 'warning' as const, emoji: '⚠️' };
      default:
        return { variant: 'info' as const, emoji: 'ℹ️' };
    }
  };

  const securityLevel = () => getSecurityLevel(metrics().securityScore);

  return (
    <Card class="border-5">
      <CardHeader>
        <div class="flex items-center justify-between">
          <CardTitle class="flex items-center gap-2">
            <span class="text-2xl">🔒</span>
            Security Status
          </CardTitle>
          <Badge class={`${securityLevel().color} border-3 flex items-center gap-2 text-white`}>
            <span class="text-lg">{securityLevel().emoji}</span>
            {securityLevel().label}
          </Badge>
        </div>
      </CardHeader>
      <CardContent class="space-y-4">
        {/* Security Score */}
        <div class="p-4 border-3 border-black bg-gradient-to-r from-primary to-secondary">
          <div class="flex items-center justify-between mb-2">
            <span class="font-bold text-sm uppercase">Security Score</span>
            <span class="text-2xl">{securityLevel().emoji}</span>
          </div>
          <div class="text-4xl font-heading font-black">
            {metrics().securityScore}/100
          </div>
          <div class="w-full bg-white h-4 border-3 border-black mt-3">
            <div
              class={`h-full ${securityLevel().color} transition-all duration-500`}
              style={{ width: `${metrics().securityScore}%` }}
            />
          </div>
        </div>

        {/* Security Metrics Grid */}
        <div class="grid grid-cols-3 gap-3">
          <div class="p-3 border-3 border-black bg-white text-center">
            <div class="text-2xl font-heading font-black text-red-600">
              {metrics().failedLoginAttempts}
            </div>
            <div class="text-xs font-bold uppercase text-neutral-darkGray">
              Failed Logins
            </div>
          </div>
          <div class="p-3 border-3 border-black bg-white text-center">
            <div class="text-2xl font-heading font-black text-green-600">
              {metrics().activeTokens}
            </div>
            <div class="text-xs font-bold uppercase text-neutral-darkGray">
              Active Tokens
            </div>
          </div>
          <div class="p-3 border-3 border-black bg-white text-center">
            <div class="text-2xl font-heading font-black text-orange-600">
              {metrics().rateLimitViolations}
            </div>
            <div class="text-xs font-bold uppercase text-neutral-darkGray">
              Rate Limits
            </div>
          </div>
        </div>

        {/* Security Features Status */}
        <div class="space-y-2">
          <h4 class="font-bold text-sm uppercase mb-3">Security Features</h4>
          
          <div class="flex items-center justify-between p-2 border-2 border-black">
            <div class="flex items-center gap-2">
              <span class="text-lg">✅</span>
              <span class="font-bold text-sm">HTTPS/TLS Encryption</span>
            </div>
            <Badge variant="success" class="border-2">Active</Badge>
          </div>

          <div class="flex items-center justify-between p-2 border-2 border-black">
            <div class="flex items-center gap-2">
              <span class="text-lg">✅</span>
              <span class="font-bold text-sm">JWT Authentication</span>
            </div>
            <Badge variant="success" class="border-2">Active</Badge>
          </div>

          <div class="flex items-center justify-between p-2 border-2 border-black">
            <div class="flex items-center gap-2">
              <span class="text-lg">✅</span>
              <span class="font-bold text-sm">Rate Limiting</span>
            </div>
            <Badge variant="success" class="border-2">Active</Badge>
          </div>

          <div class="flex items-center justify-between p-2 border-2 border-black">
            <div class="flex items-center gap-2">
              <span class="text-lg">✅</span>
              <span class="font-bold text-sm">CORS Protection</span>
            </div>
            <Badge variant="success" class="border-2">Active</Badge>
          </div>

          <div class="flex items-center justify-between p-2 border-2 border-black">
            <div class="flex items-center gap-2">
              <span class="text-lg">✅</span>
              <span class="font-bold text-sm">Input Validation</span>
            </div>
            <Badge variant="success" class="border-2">Active</Badge>
          </div>

          <div class="flex items-center justify-between p-2 border-2 border-black">
            <div class="flex items-center gap-2">
              <span class="text-lg">✅</span>
              <span class="font-bold text-sm">SQL Injection Protection</span>
            </div>
            <Badge variant="success" class="border-2">Active</Badge>
          </div>
        </div>

        {/* Recent Security Alerts */}
        <div class="space-y-2">
          <h4 class="font-bold text-sm uppercase mb-3">Recent Alerts</h4>
          <For each={alerts()}>
            {(alert) => (
              <div class="p-3 border-3 border-black bg-white">
                <div class="flex items-start gap-3">
                  <span class="text-2xl">{getAlertBadge(alert.level).emoji}</span>
                  <div class="flex-1">
                    <div class="flex items-center justify-between mb-1">
                      <span class="font-bold text-sm">{alert.title}</span>
                      <Badge 
                        variant={getAlertBadge(alert.level).variant}
                        class="border-2 text-xs"
                      >
                        {alert.level.toUpperCase()}
                      </Badge>
                    </div>
                    <p class="text-xs text-neutral-darkGray">{alert.description}</p>
                    <p class="text-xs text-neutral-darkGray mt-1">
                      {new Date(alert.timestamp).toLocaleString()}
                    </p>
                  </div>
                </div>
              </div>
            )}
          </For>
        </div>

        {/* Security Actions */}
        <div class="flex gap-2">
          <Button variant="secondary" size="sm" fullWidth class="text-xs">
            📊 View Audit Logs
          </Button>
          <Button variant="primary" size="sm" fullWidth class="text-xs bg-red-500 hover:bg-red-600">
            🚨 Security Settings
          </Button>
        </div>

        {/* Security Recommendations */}
        <div class="p-3 border-3 border-black bg-accent-yellow">
          <div class="font-bold text-sm mb-2">🔐 Security Recommendations:</div>
          <ul class="text-xs space-y-1 list-disc list-inside">
            <li>Enable 2FA for admin accounts</li>
            <li>Regularly rotate JWT secrets</li>
            <li>Monitor for suspicious activity patterns</li>
            <li>Keep dependencies updated</li>
            <li>Review access logs weekly</li>
          </ul>
        </div>
      </CardContent>
    </Card>
  );
};

export default SecurityStatus;
