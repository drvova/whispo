/// <reference types="vite/client" />

// Tauri API types are automatically available via @tauri-apps/api
// No need to define window.electron or window.api as they don't exist in Tauri

declare module '*.svg' {
  const content: string
  export default content
}

declare module '*.png' {
  const content: string
  export default content
}

declare module '*.jpg' {
  const content: string
  export default content
}
