import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/dashboard': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/img': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
    hmr: {
      port: 5173,
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: false,
  },
})
