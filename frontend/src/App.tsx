import { Component } from 'solid-js';
import { Route } from '@solidjs/router';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Clients from './pages/Clients';
import ClientDetail from './pages/ClientDetail';
import ClientCreate from './pages/ClientCreate';
import ClientEdit from './pages/ClientEdit';
import Tasks from './pages/Tasks';
import TaskCreate from './pages/TaskCreate';
import TaskEdit from './pages/TaskEdit';
import TaskDetail from './pages/TaskDetail';
import Reports from './pages/Reports';
import Notifications from './pages/Notifications';
import Files from './pages/Files';
import FileDetail from './pages/FileDetail';
import Users from './pages/Users';
import Login from './pages/Login';
import UserProfile from './pages/UserProfile';
import UserManagement from './pages/UserManagement';
import AdminDashboard from './pages/AdminDashboard';
import Analytics from './pages/Analytics';
import Search from './pages/Search';
import RbacRoles from './pages/RbacRoles';
import RbacPermissions from './pages/RbacPermissions';
import ProtectedRoute from './components/ProtectedRoute';
import AdminRoute from './components/AdminRoute';
import { RoleList, RoleDetail, RoleCreate, RoleEdit, PermissionMatrix, UserRoleList } from './features/rbac';

const App: Component = () => {
  return (
    <>
      <Route path="/login" component={Login} />

      {/* Protected user routes */}
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
        path="/clients/new"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <ClientCreate />
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
        path="/clients/:id/edit"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <ClientEdit />
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
        path="/tasks/:id"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <TaskDetail />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/tasks/new"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <TaskCreate />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/tasks/:id/edit"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <TaskEdit />
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
        path="/reports"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <Reports />
            </Layout>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/search"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <Search />
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
        path="/files/:id"
        component={() => (
          <ProtectedRoute>
            <Layout>
              <FileDetail />
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

      {/* Admin-only routes */}
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
        path="/admin/rbac/roles-legacy"
        component={() => (
          <AdminRoute>
            <Layout>
              <RbacRoles />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/admin/rbac/permissions"
        component={() => (
          <AdminRoute>
            <Layout>
              <RbacPermissions />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/admin/rbac/roles"
        component={() => (
          <AdminRoute>
            <Layout>
              <RoleList />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/admin/rbac/roles/new"
        component={() => (
          <AdminRoute>
            <Layout>
              <RoleCreate />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/admin/rbac/roles/:id"
        component={() => (
          <AdminRoute>
            <Layout>
              <RoleDetail />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/admin/rbac/roles/:id/edit"
        component={() => (
          <AdminRoute>
            <Layout>
              <RoleEdit />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/admin/rbac/matrix"
        component={() => (
          <AdminRoute>
            <Layout>
              <PermissionMatrix />
            </Layout>
          </AdminRoute>
        )}
      />
      <Route
        path="/admin/rbac/user-roles"
        component={() => (
          <AdminRoute>
            <Layout>
              <UserRoleList />
            </Layout>
          </AdminRoute>
        )}
      />
    </>
  );
};

export default App;
