import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { getCurrentUser, login as apiLogin, logout as apiLogout, type AuthUser } from '@/api/auth';

interface AuthContextValue {
  user: AuthUser | null;
  /** True only until the initial `/auth/me` settles. */
  isLoading: boolean;
  signIn: (username: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
  /** Refreshes `user` after a password change, so the default-password banner clears itself. */
  refresh: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<AuthUser | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // One call on mount restores the session after a reload. A 401 here is the
  // normal "not signed in" answer, not an error worth surfacing.
  useEffect(() => {
    getCurrentUser()
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setIsLoading(false));
  }, []);

  const signIn = async (username: string, password: string) => {
    setUser(await apiLogin(username, password));
  };

  const signOut = async () => {
    try {
      await apiLogout();
    } finally {
      // Clear locally even if the request failed — leaving a stale user on
      // screen after the user clicked Logout is worse than a lingering cookie.
      setUser(null);
    }
  };

  const refresh = async () => {
    try {
      setUser(await getCurrentUser());
    } catch {
      setUser(null);
    }
  };

  return (
    <AuthContext.Provider value={{ user, isLoading, signIn, signOut, refresh }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used inside <AuthProvider>');
  return ctx;
}
