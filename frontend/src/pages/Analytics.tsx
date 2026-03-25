import { Component } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent } from '~/components/ui';

const Analytics: Component = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Analytics</CardTitle>
      </CardHeader>
      <CardContent>
        Deferred in no-auth phase (charts/analytics will be implemented later).
      </CardContent>
    </Card>
  );
};

export default Analytics;
