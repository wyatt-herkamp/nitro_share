<template>
  <section>
    <label :for="id">Username</label>
    <input
      type="text"
      :id="id"
      v-model="value.username"
      name="Username"
      placeholder="Username"
      required
      autocomplete="username"
      ref="input"
      v-bind="$attrs"
      v-on:focusout="validateUsername"
    />
  </section>
</template>
<script setup lang="ts">
import '@/assets/styles/form.scss'
import { checkParam, CheckStatus } from '@/utils/user'
import { ref } from 'vue'
defineProps({
  id: String
})
const input = ref<HTMLInputElement | null>(null)
let value = defineModel<{
  username: string
  usernameValid: CheckStatus
}>({
  required: true
})

async function validateUsername() {
  if (value.value.username.length == 0) {
    value.value.usernameValid = CheckStatus.Empty
  } else {
    value.value.usernameValid = await checkParam({
      type: 'Username',
      content: {
        username: value.value.username
      }
    })
  }
  if (!input.value) {
    return
  }
  const inputElement = input.value as HTMLInputElement
  switch (value.value.usernameValid) {
    case CheckStatus.Ok:
      inputElement.setAttribute('data-ok', 'ok')
      break
    case CheckStatus.Invalid:
      inputElement.setAttribute('data-ok', 'error')
      break
    case CheckStatus.Taken:
      inputElement.setAttribute('data-ok', 'warn')
      break
    case CheckStatus.Empty:
      inputElement.removeAttribute('data-ok')
      break
  }
}
</script>

<style scoped lang="scss">
@import '@/assets/styles/variables.scss';
@import '@/assets/styles/form.scss';
@import './text.scss';
</style>
