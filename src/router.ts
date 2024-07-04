import { createRouter, createWebHistory } from 'vue-router';

// Import components

// Define routes
const routes = [
  {
    path: '/',
    name: 'home',
    component: () => import('./views/Home.vue'),
  },
  {
    path: '/init/:step',
    name: 'init',
    component: () => import('./views/Init.vue'),
  },
  {
    path: '/install/:step?',
    name: 'install',
    component: () => import('./views/Install.vue'),
  }
];

const router = createRouter({
  history: createWebHistory('/'),
  routes
});
export default router;
