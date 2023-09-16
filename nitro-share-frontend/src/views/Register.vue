<template>
  <main>
    <form v-on:submit.prevent="register">
      <SetUsername autofocus v-model="username" id="username" />
      <SetEmail id="email" v-model="email" />
      <Password
        id="Password"
        v-model="password.password"
        placeholder="Password"
        required
        autocomplete="new-password"
        >Password</Password
      >
      <Password
        id="PasswordC"
        v-model="password.password_confirmation"
        placeholder="Confirm Password"
        required
        autocomplete="new-password"
        >Confirm Password</Password
      >
      <SubmitButton :disabled="!validForm">Register</SubmitButton>
    </form>
  </main>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMeta } from 'vue-meta'
import router from '@/router'
import SubmitButton from '@/components/form/SubmitButton.vue'
import Password from '@/components/form/text/Password.vue'
import { CheckStatus, NewPassword } from '@/utils/user'
import http from '@/http'
import { configurationStore } from '@/stores/system'
import { notify } from '@kyvg/vue3-notification'
import SetUsername from '@/components/form/text/SetUsername.vue'
import SetEmail from '@/components/form/text/SetEmail.vue'
import { AxiosResponse } from 'axios'

const validForm = computed(() => {
  if (username.value.usernameValid != CheckStatus.Ok || email.value.emailValid != CheckStatus.Ok) {
    return false
  }
  return password.value.checkPassword()
})
useMeta({
  title: 'Register',
  meta: [
    {
      name: 'description',
      content: 'Register for the site'
    }
  ]
})

const username = ref({
  username: '',
  usernameValid: CheckStatus.Invalid
})
const email = ref({
  email: '',
  emailValid: CheckStatus.Invalid
})
const password = ref(new NewPassword())

async function register() {
  if (!validForm.value) {
    notify({
      title: 'Error',
      text: 'Invalid Form',
      type: 'error'
    })
    return
  }
  await http
    .post('/api/public/register', {
      username: username.value.username,
      password: password.value.password,
      email: email.value.email
    })
    .then((response) => {
      if (response.status == 201) {
        const systemConfiguration = configurationStore()
        if (systemConfiguration.configuration?.state.first_user) {
          systemConfiguration.load()
        }
        router.push(`/login?username=${username.value.username}`)
      }
    })
    .catch((error) => {
      if (error.response.status == 409) {
        notify({
          title: 'Error',
          text: 'Username or Email already exists',
          type: 'error'
        })
      } else {
        notify({
          title: 'Error',
          text: error.response.text,
          type: 'error'
        })
      }
    })
}
</script>

<style scoped lang="scss">
main {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
}
</style>
