<script setup>
import {ref} from 'vue'
import {rpc, saveToken} from '../lib';
import {useRouter} from 'vue-router';

const router = useRouter()

const username = ref(localStorage.getItem("username"))
const password = ref("")

async function submit() {
  if (!username.value || !password.value) {
    return
  }
  const token = await rpc('user.login', {username: username.value, password: password.value})
  saveToken(token)
  router.push({name: 'list_password'})
  localStorage.setItem('username', username.value)
}

</script>


<template>
  <v-container class="fill-height" fluid>
    <v-row>
      <v-col>
        <v-card title="登录">
          <v-card-text>
            <v-text-field v-model="username" label="用户名"></v-text-field>
            <v-text-field v-model="password" label="密码" type="password"
                          @keyup.enter="submit"></v-text-field>
          </v-card-text>
          <v-card-actions>
            <v-btn :disabled="!password" block variant="flat" @click="submit">确定</v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>