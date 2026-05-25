import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // Front talks to relative /api/... — matches prod (Nginx proxies /api).
      '/api': {
        target: 'http://127.0.0.1:5049',
        changeOrigin: true,
      },
    },
  },
})
