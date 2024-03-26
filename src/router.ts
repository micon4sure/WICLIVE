import { createRouter, createWebHistory } from 'vue-router';

// Import components
import HomeVue from './views/Home.vue';
import MapsVue from './views/Maps.vue';
// import About from './views/About.vue';

// Define routes
const routes = [
  {
    path: '/',
    name: 'Home',
    component: HomeVue,
  },
  // {
  //   path: '/maps',
  //   name: 'Maps',
  //   component: MapsVue,
  // },
];

const router = createRouter({
  history: createWebHistory('/'),
  routes
});
export default router;
