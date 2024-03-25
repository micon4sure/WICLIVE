import { createApp } from "vue";
import 'bootswatch/dist/darkly/bootstrap.min.css';
import App from "./App.vue";

import router from "./router";

const app = createApp(App)
app.use(router);
app.mount("#app");
