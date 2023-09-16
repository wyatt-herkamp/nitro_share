<template>
  <metainfo>
    <template v-slot:title="{ content }">{{
      content ? `${content} | Nitro Share` : `Nitro Share`
    }}</template>
  </metainfo>
  <Suspense>
    <!-- main content -->
    <LoadedApp />

    <!-- loading state -->
    <template #fallback>
      <div id="container">
        <h1>Loading...</h1>
      </div>
    </template>
  </Suspense>
</template>

<script setup lang="ts">
import LoadedApp from '@/LoadedApp.vue'
</script>
<style lang="scss">
@import '@/assets/styles/variables';
#container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
</style>
