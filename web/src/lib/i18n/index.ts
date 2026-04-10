import { writable, derived, get } from 'svelte/store';
import en from './en.json';
import sq from './sq.json';
import mk from './mk.json';
import sr from './sr.json';

const translations: Record<string, Record<string, string>> = { en, sq, mk, sr };

export const locale = writable('en');

export const t = derived(locale, ($locale) => {
  const dict = translations[$locale] || translations.en;
  return (key: string, fallback?: string): string => {
    return dict[key] || translations.en[key] || fallback || key;
  };
});

export const locales = [
  { code: 'en', name: 'English' },
  { code: 'sq', name: 'Shqip' },
  { code: 'mk', name: '\u041c\u0430\u043a\u0435\u0434\u043e\u043d\u0441\u043a\u0438' },
  { code: 'sr', name: 'Srpski' },
];
