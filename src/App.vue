<template>
  <div class="row flex flex-col border gap-5">
    <div>
      <h2 class="text-lg">Input Devices</h2>
      <div class="flex flex-col">
        <span v-for="device in devices.input_devices" :key="device.id">
        {{ device.name }}
        </span>
      </div>
    </div>
    <div>
      <h2 class="text-lg">Output Devices</h2>
      <div class="flex flex-col">
        <span v-for="device in devices.output_devices" :key="device.id">
        {{ device.name }}
        </span>
      </div>
    </div>
  </div>

</template>
<script lang="ts">
import Greet from "./components/Greet.vue";
import type { DeviceCollection } from "./types";
import { invoke } from '@tauri-apps/api/tauri';

export default {
  name: "App",
  components: {
    Greet,
  },
  data() {
    return {
      devices: { input_devices: [], output_devices: [] } as DeviceCollection,
    };
  },
  methods: {
    async fetchDevices() {
      try {
        const result = await invoke('get_audio_devices');
        if (typeof result === "string") {
          this.devices = JSON.parse(result);
        }
      } catch (error) {
        console.error("Failed to fetch devices:", error);
      }
    },
  },
  mounted() {
    this.fetchDevices();
  },
}

</script>
<style scoped>
</style>
