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
          <ProtectedRoute>
            <Layout>
              <Dashboard />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/clients"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <Clients />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/clients/:id"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <ClientDetail />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/tasks"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <Tasks />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/profile"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <UserProfile />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/users-management"
        component={() => (
          <ProtectedRoute>
            <AdminRoute>
              <Layout>
                <UserManagement />
              </Layout>
            </AdminRoute>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/admin-dashboard"
        component={() => (
          <ProtectedRoute>
            <AdminRoute>
              <Layout>
                <AdminDashboard />
              </Layout>
            </AdminRoute>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/analytics"
        component={() => (
          <ProtectedRoute>
            <AdminRoute>
              <Layout>
                <Analytics />
              </Layout>
            </AdminRoute>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/users"
        component={() => (
          <ProtectedRoute>
            <AdminRoute>
              <Layout>
                <Users />
              </Layout>
            </AdminRoute>
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


