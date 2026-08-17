import apiClient from './client';

export interface AuthUser {
  id: string;
  username: string;
  role: 'admin' | 'viewer';
  /** True while the account still uses the seeded default password. */
  using_default_password: boolean;
}

export async function login(username: string, password: string): Promise<AuthUser> {
  const { data } = await apiClient.post<AuthUser>('/auth/login', { username, password });
  return data;
}

export async function logout(): Promise<void> {
  await apiClient.post('/auth/logout');
}

/**
 * The session cookie is httpOnly, so JavaScript cannot read it. This endpoint
 * is the only way the SPA can tell whether it is signed in.
 */
export async function getCurrentUser(): Promise<AuthUser> {
  const { data } = await apiClient.get<AuthUser>('/auth/me');
  return data;
}

export async function changePassword(
  currentPassword: string,
  newPassword: string
): Promise<AuthUser> {
  const { data } = await apiClient.post<AuthUser>('/auth/change-password', {
    current_password: currentPassword,
    new_password: newPassword,
  });
  return data;
}
