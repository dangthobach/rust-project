import { Component } from 'solid-js';
import { Route } from '@solidjs/router';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Notifications from './pages/Notifications';
import Files from './pages/Files';
import Login from './pages/Login';
import ProtectedRoute from './components/ProtectedRoute';

const App: Component = () => {
  return (
    <>
      <Route path="/login" component={Login} />
      <Route
        path="/"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <Dashboard />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/notifications"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <Notifications />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/files"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <Files />
            </Layout>
          </ProtectedRoute>
        )}
      />
    </>
  );
};

export default App;


