import type { Component } from 'solid-js';
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui';

const UserProfile: Component = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>User Profile</CardTitle>
      </CardHeader>
      <CardContent>
        Deferred in no-auth phase. Profile will be backed by Keycloak userinfo later.
      </CardContent>
    </Card>
  );
};

export default UserProfile;

