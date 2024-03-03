<script setup>
import {onMounted, reactive, ref, watch} from 'vue';
import {rpc, toast} from '../lib';
import {mdiEye, mdiEyeOff} from '@mdi/js';
import {useRoute, useRouter} from 'vue-router';

const route = useRoute();
const router = useRouter();
const title = ref("创建密码")

const form = reactive({
  id: 0,
  name: "",
  username: "",
  password: "",
  attachment: "",
})

onMounted(async () => {
  if (route.params.id) {
    let password = await rpc('password.view', {id: parseInt(route.params.id)})
    form.id = password.id
    form.name = password.name
    form.username = password.username
    form.password = password.password
    form.attachment = password.attachment
    title.value = "编辑 " + password.name
  } else {
    generatePassword()
  }
})

const showPassword = ref(false)

const generate = reactive({
  uppercase: true,
  lowercase: true,
  digit: true,
  symbol: true,
  len: 16,
})

let generatePassword = () => {
  const chars = {
    uppercase: ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"],
    lowercase: ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"],
    digit: ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
    symbol: ['!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~'],
  }
  const kind = ['uppercase', 'lowercase', 'digit', 'symbol']

  let count = 0
  for (let v of kind) {
    count += generate[v] ? 1 : 0
  }
  if (count == 0) {
    form.password = ""
    return
  }
  const list = []
  while (generate.len > list.length) {
    let n = Math.max(parseInt((generate.len - list.length) / count), 1)
    for (let v of kind) {
      if (generate[v]) {
        list.push(...pick(chars[v], Math.min(n, generate.len - list.length)))
      }
    }
  }

  if (list.length != generate.len) {
    throw "unreachable"
  }

  shuffle(list)
  form.password = list.join("")
}

function pick(arr, n) {
  shuffle(arr)
  return arr.slice(0, n)
}

function shuffle(array) {
  for (let i = array.length - 1; i > 0; i--) {
    let j = Math.floor(Math.random() * (i + 1));
    [array[i], array[j]] = [array[j], array[i]];
  }
}

watch(generate, () => {
  generatePassword()
})


async function submit() {
  if (!form.name) {
    toast("名称不能为空")
    return
  }
  if (!form.password) {
    toast("密码不能为空")
    return
  }
  if (!form.id) {
    await rpc('password.create', form)
  } else {
    await rpc('password.update', form)
  }
  router.back()
}

</script>

<template>
  <v-layout>
    <v-app-bar density="compact" flat>
      <template v-slot:prepend>
        <v-btn icon="$back" @click="$router.back()"></v-btn>
      </template>
      <v-app-bar-title>{{ title }}</v-app-bar-title>
      <template v-slot:append>
        <v-btn icon="$check" @click="submit"></v-btn>
      </template>
    </v-app-bar>

    <v-main>
      <v-container fluid>
        <v-row>
          <v-col>
            <v-card>
              <v-card-text>
                <v-text-field v-model="form.name" label="名称"></v-text-field>
                <v-text-field v-model="form.username" label="用户名"></v-text-field>
                <v-text-field v-model="form.password" :append-inner-icon="showPassword ? mdiEye : mdiEyeOff"
                              :type="showPassword ? 'text' : 'password'"
                              label="密码"
                              @click:append-inner="showPassword = !showPassword">
                </v-text-field>
                <v-textarea v-model="form.attachment" label="附件" variant="underlined"></v-textarea>
                <v-row v-if="!$vuetify.display.xs">
                  <v-col>
                    <v-switch v-model="generate.uppercase" label="A-Z"></v-switch>
                  </v-col>
                  <v-col>
                    <v-switch v-model="generate.lowercase" label="a-z"></v-switch>
                  </v-col>
                  <v-col>
                    <v-switch v-model="generate.digit" label="0-9"></v-switch>
                  </v-col>
                  <v-col>
                    <v-switch v-model="generate.symbol" label="!@"></v-switch>
                  </v-col>
                </v-row>
                <template v-else>
                  <v-row style="height: 40px;">
                    <v-col>
                      <v-switch v-model="generate.uppercase" label="A-Z"></v-switch>
                    </v-col>
                    <v-col>
                      <v-switch v-model="generate.lowercase" label="a-z"></v-switch>
                    </v-col>
                  </v-row>
                  <v-row>
                    <v-col>
                      <v-switch v-model="generate.digit" label="0-9"></v-switch>
                    </v-col>
                    <v-col>
                      <v-switch v-model="generate.symbol" label="!@"></v-switch>
                    </v-col>
                  </v-row>
                </template>
                <v-slider v-model="generate.len" :max="128" :min="4" :step="1"
                          thumb-label="always"></v-slider>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-container>
    </v-main>
  </v-layout>
</template>