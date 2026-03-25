/**
 * Admin Dashboard Page
 * Displays system metrics, activity feed, and health status
 */

import { Component } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent } from '~/components/ui';

const AdminDashboard: Component = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Admin Dashboard</CardTitle>
      </CardHeader>
      <CardContent>
        Deferred in no-auth phase (Keycloak PKCE + admin features later).
      </CardContent>
    </Card>
  );
};

export default AdminDashboard;
