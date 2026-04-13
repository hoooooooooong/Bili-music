import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: () => import("@/pages/HomePage.vue"),
    },
    {
      path: "/downloads",
      name: "downloads",
      component: () => import("@/pages/DownloadPage.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/components/settings/SettingsPage.vue"),
    },
  ],
});

export default router;
