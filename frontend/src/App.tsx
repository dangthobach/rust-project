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
        path="/clients/new"
        component={() => (
          <Layout>
            <ClientCreate />
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
        path="/clients/:id/edit"
        component={() => (
          <Layout>
            <ClientEdit />
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
        path="/tasks/:id"
        component={() => (
          <Layout>
            <TaskDetail />
          </Layout>
        )}
      />
      <Route
        path="/tasks/new"
        component={() => (
          <Layout>
            <TaskCreate />
          </Layout>
        )}
      />
      <Route
        path="/tasks/:id/edit"
        component={() => (
          <Layout>
            <TaskEdit />
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
      {/* Legacy routes — kept for compatibility */}
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

      {/* ── RBAC feature routes ── */}
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
      <Route
        path="/reports"
        component={() => (
          <Layout>
            <Reports />
          </Layout>
        )}
      />
      <Route
        path="/search"
        component={() => (
          <Layout>
            <Search />
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
        path="/files/:id"
        component={() => (
          <Layout>
            <FileDetail />
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


