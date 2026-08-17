import { Routes, Route } from 'react-router-dom';
import {
  AnalyticsDashboard,
  Dashboard,
  GroupAnalytics,
  HotelDetail,
  HotelGroupDetail,
  HotelsList,
  Login,
  ReportView,
} from '@/pages';
import { Layout } from '@/components/layout/Layout';
import { RequireAuth } from '@/auth/RequireAuth';

function App() {
  return (
    <Routes>
      {/* The only route outside the auth gate. */}
      <Route path="/login" element={<Login />} />

      <Route
        element={
          <RequireAuth>
            <Layout />
          </RequireAuth>
        }
      >
        <Route path="/" element={<Dashboard />} />
        <Route path="/groups/:id" element={<HotelGroupDetail />} />
        <Route path="/groups/:id/analytics" element={<GroupAnalytics />} />
        <Route path="/reports/:id" element={<ReportView />} />
        <Route path="/analytics" element={<AnalyticsDashboard />} />
        <Route path="/hotels" element={<HotelsList />} />
        <Route path="/hotels/:id" element={<HotelDetail />} />
      </Route>
    </Routes>
  );
}

export default App;
