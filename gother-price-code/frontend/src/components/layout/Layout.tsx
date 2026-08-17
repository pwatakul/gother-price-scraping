import { useState } from 'react';
import { Outlet } from 'react-router-dom';
import { AlertTriangle, X } from 'lucide-react';
import { Topbar } from './Topbar';
import { Sidebar } from './Sidebar';
import { useAuth } from '@/auth/AuthContext';
import { ChangePasswordDialog } from '@/auth/ChangePasswordDialog';

/**
 * Shown while the signed-in account still uses the seeded default password.
 * Without it, `admin1234!` quietly becomes the permanent password — the banner
 * disappears on its own once the password is changed.
 */
function DefaultPasswordBanner() {
  const { user } = useAuth();
  const [dismissed, setDismissed] = useState(false);
  const [changing, setChanging] = useState(false);

  if (!user?.using_default_password || dismissed) return null;

  return (
    <>
      <div className="shrink-0 flex items-center gap-2 px-4 py-2 bg-amber-50 border-b border-amber-200 text-[13px] text-amber-900">
        <AlertTriangle className="h-4 w-4 shrink-0" />
        <span>
          This account is still using the default password. Anyone who knows it can sign in.
        </span>
        <button
          type="button"
          onClick={() => setChanging(true)}
          className="font-semibold underline underline-offset-2"
        >
          Change it now
        </button>
        <button
          type="button"
          onClick={() => setDismissed(true)}
          aria-label="Dismiss"
          className="ml-auto text-amber-700 hover:text-amber-900"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      </div>
      <ChangePasswordDialog open={changing} onOpenChange={setChanging} />
    </>
  );
}

export function Layout() {
  return (
    <div className="min-w-[1280px] h-screen overflow-hidden flex flex-col bg-background">
      <Topbar />
      <DefaultPasswordBanner />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
