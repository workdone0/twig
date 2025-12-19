// @ts-check
import { defineConfig } from 'astro/config';

import sitemap from "@astrojs/sitemap";

// https://astro.build/config
export default defineConfig({
  site: "https://workdone0.github.io",
  base: "/twig",
  integrations: [sitemap()],
});