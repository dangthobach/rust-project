import { Component } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent } from '~/components/ui';

const Login: Component = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Login</CardTitle>
      </CardHeader>
      <CardContent>
        Deferred. Auth will be implemented later with Keycloak PKCE flow.
      </CardContent>
    </Card>
  );
};

export default Login;

