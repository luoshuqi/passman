<script setup>
import { ref } from 'vue'
import { rpc, toast } from '../lib';
import { useRouter } from 'vue-router';

const router = useRouter()

const old_password = ref("")
const new_password = ref("")
const new_password1 = ref("")

async function submit() {
    if (!old_password.value || !new_password.value || !new_password1.value) {
        return
    }
    if (new_password.value !== new_password1.value) {
        toast('密码不匹配')
        return
    }
    await rpc('user.change_password', { old_password: old_password.value, new_password: new_password.value })
    router.back()
    toast('已修改')
}

</script>


<template>
    <v-layout>
        <v-app-bar density="compact" flat>
            <template v-slot:prepend>
                <v-btn icon="$back" @click="$router.back()"></v-btn>
            </template>
            <v-app-bar-title>修改密码</v-app-bar-title>

            <template v-slot:append>
                <v-btn icon="$check" @click="submit" :disabled="!old_password || !new_password || !new_password1"></v-btn>
            </template>
        </v-app-bar>
        <v-main>
            <v-container class="fill-height" fluid>
                <v-row>
                    <v-col>
                        <v-card>
                            <v-card-text>
                                <v-text-field v-model="old_password" label="当前密码" type="password"></v-text-field>
                                <v-text-field v-model="new_password" label="新密码" type="password"></v-text-field>
                                <v-text-field v-model="new_password1" label="确认新密码" type="password"
                                    @keyup.enter="submit"></v-text-field>
                            </v-card-text>
                        </v-card>
                    </v-col>
                </v-row>
            </v-container>
        </v-main>
    </v-layout>

</template>