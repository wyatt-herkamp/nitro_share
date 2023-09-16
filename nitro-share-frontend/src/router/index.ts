import { createRouter, createWebHistory } from 'vue-router'

import Index from '../views/Index.vue'
import Login from '@/views/Login.vue'
import Register from '@/views/Register.vue'

declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth: boolean
  }
}
const routes = [
  {
    path: '/',
    name: 'home',
    component: Index
  },
  {
    path: '/login',
    name: 'login',
    component: Login
  },
  {
    path: '/register',
    name: 'register',
    component: Register
  }
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes
})
export default router
