<template>
  <section>
    <label :for="id">Email</label>
    <input
      type="text"
      :id="id"
      v-model="value.email"
      name="Username"
      placeholder="Email"
      required
      autocomplete="email"
      ref="input"
      v-bind="$attrs"
      v-on:focusout="validate"
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
  email: string
  emailValid: CheckStatus
}>({
  required: true
})

async function validate() {
  if (value.value.email.length == 0) {
    value.value.emailValid = CheckStatus.Empty
  } else {
    value.value.emailValid = await checkParam({
      type: 'Email',
      content: {
        email: value.value.email
      }
    })
  }
  if (!input.value) {
    return
  }
  const inputElement = input.value as HTMLInputElement
  switch (value.value.emailValid) {
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
