<script setup lang="ts">
</script>

<template>
  <div class="maps-section">
    <h3>Maps</h3>
    <p class="text-muted">Download and manage custom maps.</p>

    <div class="maps-placeholder">
      <span class="text-dim">Map browser — coming soon</span>
    </div>
  </div>
</template>

<style scoped>
.maps-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.maps-placeholder {
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
