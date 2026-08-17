import { useState, type FormEvent } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { useAuth } from '@/auth/AuthContext';

export function Login() {
  const { user, isLoading, signIn } = useAuth();
  const location = useLocation();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const from = (location.state as { from?: string } | null)?.from ?? '/';

  // Already signed in — don't show a login form, go where they were headed.
  if (!isLoading && user) return <Navigate to={from} replace />;

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      await signIn(username, password);
      // No navigate() needed: `user` becomes non-null and the guard above
      // redirects on the next render.
    } catch (err: any) {
      setError(err?.response?.data?.error?.message ?? 'Could not sign in. Please try again.');
      setSubmitting(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-muted/40 px-4">
      <div className="w-full max-w-[380px]">
        <div className="text-center mb-6">
          <div className="text-[22px] font-extrabold text-brand-600">⚡ Gother</div>
          <div className="text-sm text-muted-foreground mt-1">Price Intelligence</div>
        </div>

        <form
          onSubmit={onSubmit}
          className="bg-background border rounded-[10px] shadow-sm px-6 py-6 space-y-4"
        >
          <h1 className="text-base font-bold">Sign in</h1>

          <div className="space-y-1.5">
            <Label htmlFor="username">Username</Label>
            <Input
              id="username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              autoComplete="username"
              autoFocus
              required
            />
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="current-password"
              required
            />
          </div>

          {error && (
            <div
              role="alert"
              className="text-[13px] text-destructive bg-destructive/10 border border-destructive/20 rounded-md px-3 py-2"
            >
              {error}
            </div>
          )}

          <Button type="submit" className="w-full" disabled={submitting}>
            {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Sign in
          </Button>
        </form>
      </div>
    </div>
  );
}
