import { Navigate, useLocation } from 'react-router-dom';
import { Loader2 } from 'lucide-react';
import { useAuth } from './AuthContext';

/**
 * Gate for every application route. While the initial `/auth/me` is in flight
 * it renders a spinner rather than redirecting — otherwise every page reload
 * would flash the login screen before landing back where you were.
 */
export function RequireAuth({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();
  const location = useLocation();

  if (isLoading) {
    return (
      <div className="h-screen flex items-center justify-center bg-background">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!user) {
    // `state.from` lets the login page send you back where you were headed.
    return <Navigate to="/login" replace state={{ from: location.pathname + location.search }} />;
  }

  return <>{children}</>;
}
