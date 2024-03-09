<script setup>
import { onMounted, ref } from 'vue'
import { rpc, toast } from '../lib.js'
import { mdiMagnify } from '@mdi/js'
import { useRouter } from 'vue-router';

const router = useRouter()

const passwords = ref([])
const filter = ref("")

const menu = [
  { title: '修改密码', handler: () => router.push({name: 'change_user_password'}) },
]

async function loadPassword() {
  passwords.value = await rpc('password.list')
}

onMounted(async () => {
  await loadPassword()
})

async function copyPassword(item) {
  let password = await rpc('password.view', { id: item.id })
  try {
    await navigator.clipboard.writeText(password.password)
    toast('已复制')
  } catch (e) {
    toast('复制失败：' + e)
  }
}

function editPassword(item) {
  router.push({ name: 'edit_password', params: { id: item.id } })
}

async function deletePassword(item) {
  if (confirm("确认删除" + item.name + "?")) {
    await rpc('password.delete', { id: item.id })
    await loadPassword()
  }
}

</script>

<template>
  <v-layout>
    <v-app-bar density="compact">
      <v-app-bar-title>
        <v-text-field v-model="filter" :prepend-inner-icon="mdiMagnify" density="compact" hide-details label="搜索"
          single-line variant="plain"></v-text-field>
      </v-app-bar-title>
      <template v-slot:append>
        <v-btn icon="$plus" @click="$router.push({ name: 'create_password' })"></v-btn>
        <v-menu>
        <template v-slot:activator="{ props }">
          <v-btn icon="$menu" v-bind="props"></v-btn>
        </template>
        <v-list>
          <v-list-item v-for="(item, index) in menu" :key="index" :value="index" @click="item.handler()">
            <v-list-item-title>{{ item.title }}</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>
      </template>
    </v-app-bar>

    <v-main>
      <v-container fluid>
        <v-row>
          <v-col>
            <v-list class="password">
              <v-list-item v-for="item in passwords" v-show="!filter || item.name.toLowerCase().includes(filter)"
                :key="item.id" :title="item.name" :value="item.id">

                <template v-slot:append>
                  <v-btn icon="$copy" size="small" variant="text" @click="copyPassword(item)"></v-btn>
                  <v-btn icon="$edit" size="small" variant="text" @click="editPassword(item)"></v-btn>
                  <v-btn icon="$delete" size="small" variant="text" @click="deletePassword(item)"></v-btn>
                </template>
              </v-list-item>
            </v-list>
          </v-col>
        </v-row>
      </v-container>
    </v-main>
  </v-layout>
</template>
