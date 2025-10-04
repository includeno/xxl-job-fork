import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api'
import router from '@/router'

export const useAuthStore = defineStore('auth', () => {
  const user = ref(null)

  async function login(username, password) {
    try {
      const response = await api.login(username, password)
      user.value = response.data
      // In a real app, you'd store a token, e.g., in localStorage
      await router.push('/')
    } catch (error) {
      console.error('Login failed:', error)
      // You could show an error message to the user here
    }
  }

  function logout() {
    user.value = null
    // In a real app, you'd remove the token from storage
    router.push('/login')
  }

  return { user, login, logout }
})