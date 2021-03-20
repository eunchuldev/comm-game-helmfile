<script lang="ts">
	import SearchBar from '../components/SearchBar.svelte';
	import HomeButton from '../components/HomeButton.svelte';
</script>

<style>
  .logo {
  }
</style>

<div class="flex flex-col items-center justify-center h-full">
  <HomeButton class="text-8xl -mt-32" />
  <SearchBar />
</div>
