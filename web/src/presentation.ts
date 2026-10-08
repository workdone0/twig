import palettes from '../../crates/twig-core/themes.json';
import bindings from '../../crates/twig-core/bindings.json';
export type ThemeName = keyof typeof palettes;
export { bindings };
export function savedTheme(): ThemeName {
  try { return localStorage.getItem('twig.theme') === 'light' ? 'light' : 'dark'; }
  catch { return 'dark'; }
}
export function applyTheme(theme: ThemeName) {
  document.documentElement.dataset.theme = theme;
  for (const [role, value] of Object.entries(palettes[theme])) {
    document.documentElement.style.setProperty(`--${role}`, value);
  }
}
export function persistTheme(theme: ThemeName) {
  try { localStorage.setItem('twig.theme', theme); return true; }
  catch { return false; }
}
export const actionFor = (key: string) => bindings.find(binding => binding.keys.includes(key))?.action;
export const keyLabel = (key: string) => ({ ArrowDown: '↓', ArrowUp: '↑', ArrowLeft: '←', ArrowRight: '→', Escape: 'Esc' }[key] ?? key);
