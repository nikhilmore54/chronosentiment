import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      // Pi RP-608 live dashboard (/api/state for Gate5 Operator)
      '/pi-dashboard': {
        target: 'http://192.168.0.9:8765',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/pi-dashboard/, ''),
      },
    },
  },
})
