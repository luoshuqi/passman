import {fileURLToPath, URL} from 'node:url'

import {defineConfig} from 'vite'
import vue from '@vitejs/plugin-vue'
import {VitePWA} from 'vite-plugin-pwa';

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        vue(),
        VitePWA({
            registerType: 'autoUpdate',
            pwaAssets: {
                image: "public/image.png"
            },
            manifest: {
                name: 'Passman',
                short_name: 'Passman',
                description: "A simple password manager.",
                start_url: "/index.html",
                // <a href="https://www.freepik.com/icon/lock_12483202#fromView=search&page=1&position=92&uuid=a945d130-3a20-4626-82c1-5c2b0c6a2503">Icon by Elite Art</a>
                icons: [{src: "/image.png", sizes: "512x512", "type": "image/png"}],
                lang: 'zh',
                theme_color: "#ffffff",
            }
        }),
    ],
    resolve: {
        alias: {
            '@': fileURLToPath(new URL('./src', import.meta.url))
        }
    },
    server: {
        proxy: {
            '/rpc': 'http://127.0.0.1:8888',
        }
    }
})
