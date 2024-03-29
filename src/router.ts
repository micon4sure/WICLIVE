import { createRouter, createWebHistory } from 'vue-router';

// Import components

// Define routes
const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('./components/views/Home.vue'),
  },
  {
    path: '/init/:step',
    name: 'Init',
    component: () => import('./components/views/Init.vue'),
  },
  {
    path: '/install/:step?',
    name: 'Install',
    component: () => import('./components/views/Install.vue'),
  }
];

const router = createRouter({
  history: createWebHistory('/'),
  routes
});
export default router;
