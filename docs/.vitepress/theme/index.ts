// https://vitepress.dev/guide/custom-theme
import { h } from 'vue'
import type { DefineComponent } from 'vue'
import type { Theme } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import './style.css'
import { enhanceAppWithTabs } from 'vitepress-plugin-tabs/client'

// import all components globally, vite provides import.meta.glob
// @ts-ignore
const modules = import.meta.glob("../components/**.vue", { eager: true }) as { [index: string]: { default: DefineComponent } }

export default {
  extends: DefaultTheme,
  Layout: () => {
    return h(DefaultTheme.Layout, null, {
      // https://vitepress.dev/guide/extending-default-theme#layout-slots
    })
  },
  enhanceApp({ app }) {
    enhanceAppWithTabs(app)
    
    // register all components globally
    for (const path in modules) {
      const component = modules[path].default;
      app.component(component.__name ?? component.name, component)
    }
  }
} satisfies Theme
