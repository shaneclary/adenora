import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export const authToken = writable<string | null>(
  browser ? localStorage.getItem('adenora_token') : null
);

export const currentUser = writable<any>(null);

authToken.subscribe((token) => {
  if (browser) {
    if (token) localStorage.setItem('adenora_token', token);
    else localStorage.removeItem('adenora_token');
  }
});

export function logout() {
  authToken.set(null);
  currentUser.set(null);
}
