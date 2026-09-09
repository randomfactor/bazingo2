import { defineConfig } from 'vitest/config'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    conditions: ['browser'],
  },
  ssr: {
    noExternal: ['svelte'],
  },
  test: {
    environment: 'jsdom',
  },
})