<script setup>
import {ref} from 'vue';
import {RouterView, useRouter} from 'vue-router'
import {on} from './lib';

const router = useRouter();

const snackbar = ref(false)
const text = ref("")
on('error', arg => {
  text.value = arg
  snackbar.value = true
})
on('invalid_token', () => {
  router.push({ name: 'login'})
})
</script>

<template>
  <RouterView/>
  <v-snackbar v-model="snackbar" :timeout="2000" color="default">
    {{ text }}
  </v-snackbar>
</template>