import {createApp} from "vue";
import devalue from '@nuxt/devalue'
import App from "./App.vue";
import {createPinia} from "pinia";
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'

const pinia = createPinia()
const app = createApp(App);
app.use(pinia);
app.use(ElementPlus)

devalue(pinia.state.value)
app.mount("#app");