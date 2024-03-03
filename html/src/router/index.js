import {createRouter, createWebHistory} from 'vue-router'
import Login from '@/views/Login.vue'
import CreateUser from '@/views/CreateUser.vue'
import List from '@/views/List.vue'
import EditPassword from '@/views/EditPassword.vue'
import {getToken} from '@/lib'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {path: '/', name: 'list_password', component: List},
        {path: '/login', name: 'login', component: Login},
        {path: '/user/create', name: 'create_user', component: CreateUser},
        {path: '/password/create', name: 'create_password', component: EditPassword},
        {path: '/password/edit/:id', name: 'edit_password', component: EditPassword},
    ]
})

router.beforeEach(to => {
    if (!getToken() && to.name != 'create_user' && to.name != 'login') {
        return {name: 'login'}
    }
})

export default router
