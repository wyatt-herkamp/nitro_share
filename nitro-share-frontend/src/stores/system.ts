import { defineStore } from 'pinia'
import { ref, Ref } from 'vue'
import http from '@/http'
import { BackendConfigurationReport } from '@/types'

export const configurationStore = defineStore(
  'configurationStore',
  () => {
    const configuration: Ref<BackendConfigurationReport | undefined> = ref(undefined)
    async function load(): Promise<BackendConfigurationReport | undefined> {
      return await http
        .get<BackendConfigurationReport>('/api/configuration')
        .then((response) => {
          console.log(`Configuration From Backend: ${JSON.stringify(response.data)}`)
          configuration.value = response.data
          return response.data
        })
        .catch(() => {
          configuration.value = undefined
          return undefined
        })
    }

    return { configuration, load }
  },
  {
    persist: true
  }
)
