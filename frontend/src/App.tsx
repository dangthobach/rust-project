import { Component } from 'solid-js';
import { Route } from '@solidjs/router';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Notifications from './pages/Notifications';
import Files from './pages/Files';
import Login from './pages/Login';

const App: Component = () => {
  return (
    <>
      <Route path="/login" component={Login} />
      <Route
        path="/"
        component={() => (
          <Layout>
            <Dashboard />
          </Layout>
        )}
      />
      <Route
        path="/notifications"
        component={() => (
          <Layout>
            <Notifications />
          </Layout>
        )}
      />
      <Route
        path="/files"
        component={() => (
          <Layout>
            <Files />
          </Layout>
        )}
      />
    </>
  );
};

export default App;


