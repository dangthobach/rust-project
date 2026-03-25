import { Component } from 'solid-js';
import { Card, CardHeader, CardTitle, CardContent } from '~/components/ui';

const ClientDetail: Component = () => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Client Detail</CardTitle>
      </CardHeader>
      <CardContent>
        Deferred in no-auth phase. We will re-implement this screen after core flows (Files/Tasks/Clients list) are stable.
      </CardContent>
    </Card>
  );
};

export default ClientDetail;