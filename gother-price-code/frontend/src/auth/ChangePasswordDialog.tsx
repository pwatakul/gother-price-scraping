import { useState, type FormEvent } from 'react';
import { Loader2 } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/Dialog';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { changePassword } from '@/api/auth';
import { useAuth } from './AuthContext';

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ChangePasswordDialog({ open, onOpenChange }: Props) {
  const { refresh } = useAuth();
  const [current, setCurrent] = useState('');
  const [next, setNext] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const reset = () => {
    setCurrent('');
    setNext('');
    setConfirm('');
    setError(null);
    setSaving(false);
  };

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);

    // Checked here as well as on the server so a typo in the confirmation
    // doesn't silently set a password the user didn't intend.
    if (next !== confirm) {
      setError('The two new passwords do not match.');
      return;
    }

    setSaving(true);
    try {
      await changePassword(current, next);
      // Refresh so `using_default_password` — and the banner — updates.
      await refresh();
      reset();
      onOpenChange(false);
    } catch (err: any) {
      setError(err?.response?.data?.error?.message ?? 'Could not change the password.');
      setSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o) reset();
        onOpenChange(o);
      }}
    >
      <DialogContent className="sm:max-w-[420px]">
        <DialogHeader>
          <DialogTitle>Change password</DialogTitle>
          <DialogDescription>
            Existing sessions in other browsers stay valid until they expire.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={onSubmit} className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="current-password">Current password</Label>
            <Input
              id="current-password"
              type="password"
              value={current}
              onChange={(e) => setCurrent(e.target.value)}
              autoComplete="current-password"
              required
            />
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="new-password">New password</Label>
            <Input
              id="new-password"
              type="password"
              value={next}
              onChange={(e) => setNext(e.target.value)}
              autoComplete="new-password"
              minLength={8}
              required
            />
            <p className="text-xs text-muted-foreground">At least 8 characters.</p>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="confirm-password">Confirm new password</Label>
            <Input
              id="confirm-password"
              type="password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              autoComplete="new-password"
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

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={saving}>
              {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Change password
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
