// Static SPA: `nuxt generate` writes .output/public, served by the Worker's static assets.
export default defineNuxtConfig({
  compatibilityDate: '2026-11-01',
  ssr: false,
  app: {
    head: {
      title: 'Rust Server Status',
      htmlAttrs: { lang: 'ja' },
      meta: [{ name: 'viewport', content: 'width=device-width, initial-scale=1' }],
    },
  },
})
