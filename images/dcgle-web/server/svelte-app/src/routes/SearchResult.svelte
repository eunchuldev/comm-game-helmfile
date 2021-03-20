<script lang="ts">
	import SearchBar from '../components/SearchBar.svelte';
	import HomeButton from '../components/HomeButton.svelte';
  import { operationStore, query } from "@urql/svelte";
  import moment from 'moment';

  export let location;

  export let kind = 'document';
  export let galleryId = '';
  export let lastCreatedAt = '';
  export let searchField = '';
  export let search = '';

  $: queryParams = new URLSearchParams(location.search);
  $: if(queryParams) {
    kind = queryParams.get('kind');
    galleryId = queryParams.get('galleryId');
    search = queryParams.get('search');
    lastCreatedAt = queryParams.get('lastCreatedAt');
    searchField = queryParams.get('searchField');
  }

  let documents = operationStore(`
    query ($galleryId: String, $galleryName: String, $title: String, $authorNickname: String, $authorIp: String, $authorId: String, $lastCreatedAt: DateTimeUtc) {
      documents (galleryId: $galleryId, title: $title, authorNickname: $authorNickname, authorIp: $authorIp, authorId: $authorId, lastCreatedAt: $lastCreatedAt) {
        galleryId
        galleryName
        id
        title
        authorId
        authorNickname
        authorIp
        likeCount
        viewCount
        kind
        isRecommend
        createdAt
      }
    }`,
    {
      galleryId, 
      lastCreatedAt, 
      title: searchField == 'titleOrContents'? search: undefined,
      [searchField]: search,
    }
  );
  query(documents); 
</script>

<style>
  a {
    @apply hover:underline cursor-pointer;
  }
  a:visited {
    color: purple;
  }
</style>

<div class="flex pt-2 pb-4 border-b items-cetner">
  <SearchBar class="mx-6" search={search} galleryId={galleryId} lastCreatedAt={lastCreatedAt} searchField={searchField} />
</div>
<div class="md:flex md:container mx-auto p-6 items-start">
  <div class="md:w-1/2 w-full p-8 border"> 
    <h2 class="text-2xl mb-4"> 게시글 </h2>
    <ul class="">
      {#if $documents.fetching}
        Loading...
      {:else if $documents.error}
        Oh no! {$documents.error.message}
      {:else if !$documents.data}
        No data
      {:else}
      {#each $documents.data.documents as doc}
        <li class="py-3"> 
          <span> 
            <a href="https://gall.dcinside.com/{doc.galleryId}/{doc.docId}"> {doc.title} </a> 
          </span>
          <div class="text-sm text-gray-500 dark:text-gray-400 tracking-wide">
            <span> {doc.galleryId} </span> |
            <span> 
              {#if doc.authorIp } 
                {doc.authorNickname}({doc.authorIp})
              {:else}
                {doc.authorNickname}(고)
              {/if}
            </span> |
            <span>
              {moment(doc.createdAt).fromNow()} 
            </span>
          </div>
        </li>
      {/each}
      {/if}
    </ul>
  </div>
  <div class="md:w-1/2 p-8 border md:ml-6 mt-8 md:mt-0">
    <h2 class="text-2xl mb-4"> 댓글 </h2>
    <ul class="">
  </div>
</div>
