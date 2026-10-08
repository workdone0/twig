import { defineConfig } from 'vite';
export default defineConfig(({ command }) => ({
  // Vite injects styles and uses a WebSocket in development. Production retains
  // the strict, self-only policy from index.html.
  plugins: command === 'serve' ? [{
    name: 'development-csp',
    transformIndexHtml: (html: string) => html.replace(/<meta http-equiv="Content-Security-Policy"[^>]*\/>/, ''),
  }] : [],
}));
