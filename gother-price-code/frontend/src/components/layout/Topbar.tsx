import { useState } from 'react';
import { KeyRound, LogOut } from 'lucide-react';
import { useAuth } from '@/auth/AuthContext';
import { ChangePasswordDialog } from '@/auth/ChangePasswordDialog';

/** Two initials from a username, for the avatar circle. */
function initials(username: string): string {
  return username.slice(0, 2).toUpperCase();
}

export function Topbar() {
  const { user, signOut } = useAuth();
  const [changingPassword, setChangingPassword] = useState(false);

  return (
    <header className="h-[52px] shrink-0 bg-topbar text-white flex items-center px-4">
      <div className="flex items-baseline gap-2">
        <span className="text-[15px] font-extrabold text-brand-400">⚡ Gother</span>
        <span className="text-[13px] text-white/50">Price Intelligence</span>
      </div>
      <div className="ml-auto flex items-center gap-3 text-[12px] text-white/60">
        <span>v0.1 · Bangkok</span>

        {user && (
          <>
            <span className="text-white/80">{user.username}</span>
            {/* The only place roles are visible today — they are stored and
                carried end to end, but nothing is restricted by them yet. */}
            <span className="text-[10px] uppercase tracking-wide font-semibold rounded-full px-2 py-0.5 bg-white/10 text-white/70">
              {user.role}
            </span>

            <button
              type="button"
              onClick={() => setChangingPassword(true)}
              title="Change password"
              className="flex items-center gap-1 rounded-[7px] px-2 py-1 hover:bg-white/10 hover:text-white transition-colors"
            >
              <KeyRound className="h-3.5 w-3.5" />
            </button>

            <button
              type="button"
              onClick={signOut}
              className="flex items-center gap-1 rounded-[7px] px-2 py-1 hover:bg-white/10 hover:text-white transition-colors"
            >
              <LogOut className="h-3.5 w-3.5" />
              Logout
            </button>

            <div className="h-7 w-7 rounded-full bg-slate-700 flex items-center justify-center text-[11px] font-semibold text-white">
              {initials(user.username)}
            </div>
          </>
        )}
      </div>

      <ChangePasswordDialog open={changingPassword} onOpenChange={setChangingPassword} />
    </header>
  );
}
