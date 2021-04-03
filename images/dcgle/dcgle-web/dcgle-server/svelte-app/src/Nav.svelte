<script lang="ts">
  import { Router, Link, Route } from "svelte-routing";
  import Home from "./routes/Home.svelte";
  import About from "./routes/About.svelte";
  import Blog from "./routes/Blog.svelte";

  export let url = "";
</script>

<style>
	.custom-style {
		@apply italic;
	}
</style>
