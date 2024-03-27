import { createRouter, createWebHistory } from 'vue-router';

// Import components

// Define routes
const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('./views/Home.vue'),
  },
  {
    path: '/init',
    name: 'Init',
    component: () => import('./views/Init.vue'),
  },
  {
    path: '/install',
    name: 'Install',
    component: () => import('./views/Install.vue'),
  }
];

const router = createRouter({
  history: createWebHistory('/'),
  routes
});
export default router;
