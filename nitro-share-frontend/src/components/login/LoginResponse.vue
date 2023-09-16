<template>
  <div v-if="success">
    <h1>Logged In</h1>
    <p>Welcome {{ success.user.name }}</p>
  </div>
  <div v-else>
    <h1>Logging In</h1>
    <Spinner />
  </div>
</template>

<script setup lang="ts">
import type { Ref } from 'vue'
import { ref, watch } from 'vue'
import Spinner from '@/components/spinner/Spinner.vue'
import http from '@/http'
import { sessionStore } from '@/stores/session'
import router from '@/router'
import { LoginResponse, Session } from '@/types'
const store = sessionStore()
const emit = defineEmits(['onFail'])
const success: Ref<LoginResponse | undefined> = ref(undefined)
const props = defineProps<{
  username: string
  password: string
  redirect: string
}>()
async function callRedirect() {
  await new Promise((resolve) => setTimeout(resolve, 1000))
  await router.push(props.redirect)
}
watch(success, (value) => {
  if (value) {
    callRedirect()
  }
})
async function login() {
  await new Promise((resolve) => setTimeout(resolve, 1000))
  await http
    .post<LoginResponse>('/api/public/login', {
      username: props.username,
      password: props.password
    })
    .then((response) => {
      success.value = response.data
      store.login(response.data.session as Session, response.data.user)
    })
    .catch(() => {
      emit('onFail')
    })
}
login()
</script>
<style scoped lang="scss"></style>
