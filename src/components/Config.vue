<script setup lang="ts">
</script>

<template>
  <div class="config-section">
    <h3>Configuration</h3>
    <p class="text-muted">wicautoexec.txt editor and game config presets.</p>

    <div class="config-placeholder">
      <span class="text-dim">Config editor — coming soon</span>
    </div>
  </div>
</template>

<style scoped>
.config-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.config-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  background: rgba(var(--s1-rgb), 0.4);
  border: 1px dashed var(--bd);
  font-family: 'Oswald', sans-serif;
  text-transform: uppercase;
  letter-spacing: 1px;
  font-size: 14px;
}
</style>
