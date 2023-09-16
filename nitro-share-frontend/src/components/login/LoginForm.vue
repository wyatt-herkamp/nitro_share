<template>
  <h1>Login</h1>
  <div v-if="tryAgain" id="tryAgain">
    <h3>Incorrect Username or Password</h3>
  </div>
  <div v-if="config.configuration && config.configuration.state.first_user">
    <h3>First User</h3>
    <p>
      Please go to the <router-link to="/register">register</router-link> page to create the first
      user.
    </p>
  </div>
  <form v-on:submit="$emit('login', form)">
    <TextInput
      id="username"
      name="Username"
      v-model="form.username"
      placeholder="Username Or Email Address"
      required
      autocomplete="username"
      autofocus
      >Username</TextInput
    ><Password
      id="Password"
      v-model="form.password"
      placeholder="Password"
      required
      autocomplete="current-password"
      >Password</Password
    >
    <h6 id="reset-password">
      Forgot Password? <router-link to="reset-password">Click Here</router-link>
    </h6>
    <SubmitButton>Login</SubmitButton>
  </form>
</template>
<script setup lang="ts">
import { Ref, ref } from 'vue'
import TextInput from '@/components/form/text/TextInput.vue'
import Password from '@/components/form/text/Password.vue'
import SubmitButton from '@/components/form/SubmitButton.vue'
import { configurationStore } from '@/stores/system'
interface Form {
  username: string
  password: string
}
defineEmits<{
  (event: 'login', form: Form): void
}>()
const props = defineProps<{
  username?: string
  tryAgain: boolean
}>()
const config = configurationStore()

const form: Ref<Form> = ref({
  username: props.username ? props.username : '',
  password: ''
})
</script>
<style scoped lang="scss">
#tryAgain {
  text-align: center;
  color: #fff;
  font-weight: bold;
}
#reset-password {
  text-align: center;
  color: #fff;
  font-weight: bold;
  a {
    color: #fff;
    font-weight: bold;
    text-decoration: none;
    &:hover {
      color: #00bd7e;
      transition: color;
      transition-duration: 0.5s;
      transition-timing-function: ease-in-out;
    }
  }
}
form {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 1rem;
}
</style>
