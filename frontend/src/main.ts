import 'normalize.css/normalize.css'
import './assets/styles/main.scss'
import 'vue-final-modal/style.css'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import Notifications from '@kyvg/vue3-notification'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createVfm } from 'vue-final-modal'
import App from './App.vue'
import router from './router'
import { createMetaManager } from 'vue-meta'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { library } from '@fortawesome/fontawesome-svg-core'

const app = createApp(App)
const vfm = createVfm()
app.use(router)

import {
  faCalendar,
  faFileImage,
  faFileText,
  faGear,
  faUser,
  faBars,
  faX,
  faRightToBracket
} from '@fortawesome/free-solid-svg-icons'
import { sessionStore } from '@/stores/session'

/* add icons to the library */
library.add(faGear)
library.add(faUser)
library.add(faBars)
library.add(faFileText)
library.add(faFileImage)
library.add(faCalendar)
library.add(faRightToBracket)
library.add(faX)

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)
app.use(createMetaManager())
app.use(pinia)
app.component('font-awesome-icon', FontAwesomeIcon)

router.beforeEach((to) => {
  const store = sessionStore(pinia)
  if (to.meta.requiresAuth && store.session === undefined) {
    return {
      name: 'login'
    }
  }
})
app.use(Notifications)

app.use(vfm)
app.mount('#app')
