import { Routes, Route } from 'react-router-dom';
import { Dashboard, HotelGroupDetail, ReportView } from '@/pages';

function App() {
  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-white sticky top-0 z-50">
        <div className="container mx-auto px-4 h-16 flex items-center">
          <a href="/" className="flex items-center gap-2">
            <div className="w-8 h-8 bg-primary rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-sm">HP</span>
            </div>
            <span className="font-semibold text-lg">Hotel Price Scraper</span>
          </a>
          <div className="ml-auto text-sm text-muted-foreground">
            Gother Challenge 2026
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/groups/:id" element={<HotelGroupDetail />} />
          <Route path="/reports/:id" element={<ReportView />} />
        </Routes>
      </main>

      {/* Footer */}
      <footer className="border-t mt-auto py-6">
        <div className="container mx-auto px-4 text-center text-sm text-muted-foreground">
          Hotel Price Scraper — Built for Gother Challenge
        </div>
      </footer>
    </div>
  );
}

export default App;
