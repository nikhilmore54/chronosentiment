/// <reference types="vitest" />
import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig(() => {
  const env = loadEnv('', process.cwd())
const apiTarget = env.VITE_API_BASE_URL ? `${env.VITE_API_BASE_URL}` : `http://${process.env.HOSTNAME || '0.0.0.0'}:3001`;
  return {
    plugins: [react()],
    server: {
      host: true,
      port: 5173,
      proxy: {
        // Proxy API calls to the Rust backend using env var
        '/api': {
          target: apiTarget,
          changeOrigin: true,
          secure: false,
        },
      },
    },
    test: {
      globals: true,
      environment: 'node',
      include: ['src/**/*.test.ts', 'src/**/*.test.tsx'],
    },
  }
})
