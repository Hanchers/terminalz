<template>
  <div id="app-container">
    <Terminal />
  </div>
</template>

<script setup>
import Terminal from "./components/Terminal.vue";
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
html,
body,
#app,
#app-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
}
body {
  background: #1e1e1e;
  color: #fff;
  font-family: monospace;
}
</style>
