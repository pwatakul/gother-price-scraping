import { Routes, Route } from 'react-router-dom';
import { AnalyticsDashboard, Dashboard, HotelDetail, HotelGroupDetail, HotelsList, ReportView } from '@/pages';
import { Layout } from '@/components/layout/Layout';

function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/groups/:id" element={<HotelGroupDetail />} />
        <Route path="/reports/:id" element={<ReportView />} />
        <Route path="/analytics" element={<AnalyticsDashboard />} />
        <Route path="/hotels" element={<HotelsList />} />
        <Route path="/hotels/:id" element={<HotelDetail />} />
      </Route>
    </Routes>
  );
}

export default App;
