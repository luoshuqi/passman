<script setup>
import {ref} from 'vue'
import {rpc, toast} from '../lib';
import {useRouter} from 'vue-router';

const router = useRouter()

const username = ref("")
const password = ref("")
const password1 = ref("")

async function submit() {
  if (!username.value || !password.value || !password1.value) {
    return
  }
  if (password.value !== password1.value) {
    toast("密码不匹配")
    return
  }
  await rpc('user.create', {username: username.value, password: password.value})
  router.push({name: 'login'})
}

</script>

<template>
  <v-container class="fill-height" fluid>
    <v-row>
      <v-col>
        <v-card title="创建用户">
          <v-card-text>
            <v-text-field v-model="username" label="用户名"></v-text-field>
            <v-text-field v-model="password" label="密码" type="password"></v-text-field>
            <v-text-field v-model="password1" label="确认密码" type="password"
                          @keyup.enter="submit"></v-text-field>
          </v-card-text>
          <v-card-actions>
            <v-btn :disabled="!username || !password || !password1" block variant="flat"
                   @click="submit">确定
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>