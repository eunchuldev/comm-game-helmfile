<script lang="ts">
	import ModeSwitcher from './ModeSwitcher.svelte';
	import Tailwindcss from './Tailwindcss.svelte';

  import { Router, Link, Route } from "svelte-routing";
  import Home from "./routes/Home.svelte";
  import About from "./routes/About.svelte";
  import Blog from "./routes/Blog.svelte";

  export let url = "";
</script>

<style>
  nav > :global(*) {
    @apply mr-2
  }

  :global(.dark input) {
    @apply border-white bg-gray-700 text-white
  }
  :global(.dark select) {
    @apply border-white bg-gray-700 text-white
  }
  :global(.dark svg) {
    fill: white;
  }
</style>

<Tailwindcss />
<Router url="{url}">
  <div class="flex flex-col h-full">
    <nav class="self-end flex items-center">
      <Link to="/">Home</Link>
      <Link to="about">About</Link>
      <Link to="blog">Blog</Link>
      <ModeSwitcher />
    </nav>
    <div class="flex-grow w-full">
      <Route path="blog" component="{Blog}" />
      <Route path="about" component="{About}" />
      <Route path="/"><Home /></Route>
    </div>
  </div>
</Router>
