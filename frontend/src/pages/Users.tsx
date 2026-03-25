import type { Component } from 'solid-js';
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui';

const Users: Component = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Users</CardTitle>
      </CardHeader>
      <CardContent>
        Deferred in no-auth phase. Admin user management will be implemented after Keycloak PKCE integration.
      </CardContent>
    </Card>
  );
};

export default Users;

