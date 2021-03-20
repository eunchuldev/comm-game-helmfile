<script lang="ts">
	import ModeSwitcher from './ModeSwitcher.svelte';
	import Tailwindcss from './Tailwindcss.svelte';
	import HomeButton from './components/HomeButton.svelte';

  import { Router, Link, Route } from "svelte-routing";
  import Home from "./routes/Home.svelte";
  import About from "./routes/About.svelte";
  import Blog from "./routes/Blog.svelte";
  import SearchResult from "./routes/SearchResult.svelte";

  import { initClient } from "@urql/svelte";

  import moment from 'moment';
  import 'moment/locale/ko'
  moment.locale('ko')


  export let url = "";
  export let graphqlUrl = "/graphql";

  initClient({
    url: graphqlUrl,
  });
</script>

<style>
  nav > :global(*) {
    @apply mr-2;
  }

  :global(div, ul) {
    @apply rounded-md border-carbon-200;
  }

  :global(.dark *) {
    @apply border-carbon-500;
  }

  :global(button) {
    @apply rounded-3xl border border-carbon-200;
    box-shadow: 0 1px 3px rgb(32 33 36 / 28%);
  }
  :global(select, input) {
    @apply rounded-md bg-white border border-carbon-200;
  }


  :global(.dark button) {
    @apply bg-carbon-600 border-carbon-500;
  }
  :global(.dark input, .dark select) {
    @apply border-carbon-500 bg-carbon-700 text-white;
  }
  :global(.dark select option) {
    @apply bg-carbon-700 text-white;
  }
  :global(.dark svg) {
    fill: white;
  }


</style>

<Tailwindcss />
<Router url="{url}">
  <div class="flex flex-col h-full">
    <nav class="self-start flex items-center w-full">
      <HomeButton class="text-2xl mx-4 px-2 mr-6" />
      <Link to="/">Home</Link>
      <Link to="about">About</Link>
      <Link to="blog">Blog</Link>
      <ModeSwitcher class="ml-auto"/>
    </nav>
    <div class="flex-grow w-full">
      <Route path="blog" component="{Blog}" />
      <Route path="about" component="{About}" />
      <Route path="search-result" component="{SearchResult}" />
      <Route path="/"><Home /></Route>
    </div>
  </div>
</Router>
