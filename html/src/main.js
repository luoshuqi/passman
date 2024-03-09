import './assets/main.css'

import {createApp} from 'vue'
import App from './App.vue'
import router from './router'

// Vuetify
import 'vuetify/styles'
import {createVuetify} from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import {aliases, mdi} from 'vuetify/iconsets/mdi-svg'
import {mdiArrowLeft, mdiCheck, mdiContentCopy, mdiPencil, mdiPlus, mdiTrashCan, mdiMenu} from '@mdi/js';

const vuetify = createVuetify({
    defaults: {
        global: {
            color: 'info',
        },
        VAppBar: {
            color: 'default',
            elevation: 0,
        },
        VTextField: {
            variant: 'underlined'
        },
        VCard: {
            variant: 'flat',
            color: 'default',
        },
        VBtn: {
            size: 'large',
            variant: 'flat',
        }
    },
    icons: {
        defaultSet: 'mdi',
        sets: {
            mdi,
        },
        aliases: {
            ...aliases,
            "back": mdiArrowLeft,
            "check": mdiCheck,
            "plus": mdiPlus,
            "edit": mdiPencil,
            "delete": mdiTrashCan,
            "copy": mdiContentCopy,
            "menu": mdiMenu,
        }
    },
    aliases,
    components,
    directives,
})


const app = createApp(App)

app.use(router).use(vuetify)

app.mount('#app')
