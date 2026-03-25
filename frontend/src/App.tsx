import { Component } from 'solid-js';
import { Route } from '@solidjs/router';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Clients from './pages/Clients';
import ClientDetail from './pages/ClientDetail';
import Tasks from './pages/Tasks';
import Notifications from './pages/Notifications';
import Files from './pages/Files';
import Users from './pages/Users';
import Login from './pages/Login';
import UserProfile from './pages/UserProfile';
import UserManagement from './pages/UserManagement';
import AdminDashboard from './pages/AdminDashboard';
import Analytics from './pages/Analytics';
import ProtectedRoute from './components/ProtectedRoute';
import AdminRoute from './components/AdminRoute';

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
        path="/clients"
        component={() => (
          <Layout>
            <Clients />
          </Layout>
        )}
      />
      <Route
        path="/clients/:id"
        component={() => (
          <Layout>
            <ClientDetail />
          </Layout>
        )}
      />
      <Route
        path="/tasks"
        component={() => (
          <Layout>
            <Tasks />
          </Layout>
        )}
      />
      <Route
        path="/profile"
        component={() => (
          <Layout>
            <UserProfile />
          </Layout>
        )}
      />
      <Route
        path="/users-management"
        component={() => (
          <AdminRoute>
            <Layout>
              <UserManagement />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/admin-dashboard"
        component={() => (
          <AdminRoute>
            <Layout>
              <AdminDashboard />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/analytics"
        component={() => (
          <AdminRoute>
            <Layout>
              <Analytics />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/users"
        component={() => (
          <AdminRoute>
            <Layout>
              <Users />
            </Layout>
          </AdminRoute>
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


