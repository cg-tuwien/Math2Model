import { createRouter, createWebHashHistory, type Router } from "vue-router";
import NotFoundView from "./NotFoundView.vue";

const router: Router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: () => import("./HomeView.vue"),
    },
    { path: "/:pathMatch(.*)*", name: "NotFound", component: NotFoundView },
  ],
});

export default router;
