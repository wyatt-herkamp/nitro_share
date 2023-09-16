import { defineStore } from 'pinia'
import { Ref, ref } from 'vue'
import http from '@/http'
import { Session, User } from '@/types'

export const sessionStore = defineStore(
  'session',
  () => {
    const session: Ref<Session | undefined> = ref(undefined)
    const user: Ref<User | undefined> = ref(undefined)

    function login(s: Session, u: User) {
      session.value = s
      user.value = u
    }
    async function logout() {
      await http
        .get('/api/me/logout')
        .then(() => {})
        .catch(() => {})

      session.value = undefined
      user.value = undefined
    }
    async function updateUser(): Promise<User | undefined> {
      if (session.value == undefined) {
        return undefined
      }
      // Check if the session is still valid
      if (session.value.expires < new Date()) {
        session.value = undefined
        user.value = undefined
        return undefined
      }

      return await http
        .get<User>('/api/me')
        .then((response) => {
          console.log(`The user is still logged in: ${JSON.stringify(response.data)}`)
          user.value = response.data
          return response.data
        })
        .catch(() => {
          user.value = undefined
          session.value = undefined
          return undefined
        })
    }

    return { session, account: user, login, updateUser, logout }
  },
  {
    persist: {
      afterRestore: (data) => {
        data.store.updateUser()
      }
    }
  }
)
