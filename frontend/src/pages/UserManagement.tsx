import type { Component } from 'solid-js';
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui';

const UserManagement: Component = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>User Management</CardTitle>
      </CardHeader>
      <CardContent>
        Deferred in no-auth phase. This will be re-enabled with Keycloak PKCE + RBAC.
      </CardContent>
    </Card>
  );
};

export default UserManagement;

