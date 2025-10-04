import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../views/LoginView.vue'
import MainLayout from '../views/MainLayout.vue'
import DashboardView from '../views/DashboardView.vue'
import JobManagementView from '../views/JobManagementView.vue'
import ExecutorManagementView from '../views/ExecutorManagementView.vue'
import JobLogView from '../views/JobLogView.vue'
import UserManagementView from '../views/UserManagementView.vue'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: LoginView,
    },
    {
      path: '/',
      component: MainLayout,
      redirect: '/dashboard',
      children: [
        {
          path: 'dashboard',
          name: 'dashboard',
          component: DashboardView,
        },
        {
          path: 'jobs',
          name: 'jobs',
          component: JobManagementView,
        },
        {
          path: 'job-groups',
          name: 'job-groups',
          component: ExecutorManagementView,
        },
        {
          path: 'logs',
          name: 'logs',
          component: JobLogView,
        },
        {
          path: 'users',
          name: 'users',
          component: UserManagementView,
        },
      ],
    },
  ],
})

router.beforeEach((to, from, next) => {
  // This will fail when the app first loads because the store is not initialized yet.
  // We will address this later. For now, this sets up the basic structure.
  const authStore = useAuthStore()
  const isAuthenticated = authStore.user !== null

  if (to.name !== 'login' && !isAuthenticated) {
    next({ name: 'login' })
  } else if (to.name === 'login' && isAuthenticated) {
    next({ name: 'dashboard' })
  } else {
    next()
  }
})

export default router