import { defineStore } from 'pinia'

export const useAppStore = defineStore('app', {
  state: () => ({
    name: 'Velix',
    version: '0.1.0',
  }),
})
