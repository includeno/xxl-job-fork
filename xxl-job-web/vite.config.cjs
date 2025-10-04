const { defineConfig } = require('vite')
const vue = require('@vitejs/plugin-vue')
const vueDevTools = require('vite-plugin-vue-devtools')
const path = require('path')

// https://vite.dev/config/
module.exports = defineConfig({
  plugins: [
    vue(),
    vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    },
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})