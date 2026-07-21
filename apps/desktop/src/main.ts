import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { getSettings, onSettings } from '@velix/core'
import App from './App.vue'
import TrayMenu from './tray/TrayMenu.vue'
import { applyTheme } from './theme'
import './style.css'

// One bundle serves both windows; the tray popup is the same app at ?view=tray.
const isTray = new URLSearchParams(window.location.search).get('view') === 'tray'

if (isTray) {
  // The popup window is transparent so the panel can draw its own shadow.
  document.documentElement.style.background = 'transparent'
  document.body.style.background = 'transparent'

  // The popup has no settings store, but still has to follow the theme.
  getSettings()
    .then((settings) => applyTheme(settings.theme))
    .catch(() => {})
  onSettings((settings) => applyTheme(settings.theme))
}

createApp(isTray ? TrayMenu : App)
  .use(createPinia())
  .mount('#app')
