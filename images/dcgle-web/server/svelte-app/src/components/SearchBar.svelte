<script lang="ts">
  import { navigate } from "svelte-routing";
  const GalleryOptions = [
    { id: '', text: '전체 갤러리' },
  ];
  const SearchFieldOptions = [
		//{ id: 'titleOrContents', text: `제목/댓글` },
		{ id: 'authorNickname', text: `닉네임` },
		{ id: 'authorId', text: `아이디` },
		{ id: 'authorIp', text: `아이피` },
	];
  export let kind = 'document';
  export let galleryId = GalleryOptions[0].id;
  export let search = '';
  export let lastCreatedAt = '';
  export let searchField = SearchFieldOptions[0].id;

  function submit() {
    let params = { search, galleryId, kind, searchField, lastCreatedAt };
    navigate('/search-result?' + new URLSearchParams(params).toString());
  }

</script>

<style>
  .shapeless {
    @apply shadow-none bg-transparent border-transparent 
  }
</style>

<form class="{$$props.class} -mt-2" on:submit={submit}>
  <div class="inline-block mt-2">
    <label for="galleryId" class="sr-only"> 갤러리 </label>
        <select id="galleryId" name="galleryId" bind:value={galleryId} class="py-2 pl-3 pr-2 w-full">
          {#each GalleryOptions as opt}
            <option value={opt.id} class="tracking-wide">
              {opt.text}
            </option>
          {/each}
        </select>
  </div>
  <div class="inline-block mt-2">
    <label for="title" class="block"> 
    </label>
    <div class="relative">
      <div class="">
        <input id="search" name="search" bind:value={search} class="py-2 px-3 pr-32 white w-96" />
      </div>
      <div class="absolute inset-y-0 right-0 flex items-center">
        <label for="searchField" class="sr-only"> 종류 </label>
        <select id="searchField" name="searchField" bind:value={searchField} class="py-2 px-1 sm:text-sm shapeless">
          {#each SearchFieldOptions as opt}
            <option value={opt.id} class="tracking-wide">
              {opt.text}
            </option>
          {/each}
        </select>
        <a class="py-2 px-2 mr-1 cursor-pointer" on:click="{submit}"> 
          <svg width="1.5rem" height="1.5rem" 
            xmlns="http://www.w3.org/2000/svg" version="1.0"
            viewBox="0 0 1280.000000 1280.000000" preserveAspectRatio="xMidYMid meet">
            <metadata>
            Created by potrace 1.15, written by Peter Selinger 2001-2017
            </metadata>
            <g transform="translate(0.000000,1280.000000) scale(0.100000,-0.100000)" stroke="none">
            <path d="M4470 11700 c-1261 -110 -2368 -843 -2959 -1961 -414 -785 -529 -1723 -315 -2584 237 -951 844 -1771 1688 -2278 1121 -673 2523 -695 3659 -59 119 67 333 204 402 258 l30 24 1965 -1964 c1529 -1528 1977 -1970 2020 -1994 199 -110 430 -72 600 98 169 169 208 403 98 599 -24 44 -466 492 -1994 2021 l-1964 1965 24 30 c54 69 189 279 257 400 427 762 564 1652 388 2517 -115 570 -355 1077 -740 1567 -103 131 -411 441 -539 543 -570 452 -1194 718 -1890 803 -150 19 -583 27 -730 15z m592 -925 c953 -101 1780 -684 2192 -1545 115 -241 187 -467 231 -725 135 -793 -71 -1588 -574 -2212 -96 -120 -295 -318 -414 -413 -412 -329 -897 -529 -1432 -591 -137 -16 -456 -16 -595 0 -529 60 -1024 266 -1440 597 -109 87 -327 305 -414 414 -311 390 -508 848 -583 1355 -25 171 -25 593 0 755 51 330 131 585 269 862 323 647 891 1142 1573 1372 193 66 426 116 620 135 145 14 419 12 567 -4z"/>
            </g>
          </svg>
        </a>
      </div>
    </div>
  </div>
</form>
<div class="mt-6">
</div>
