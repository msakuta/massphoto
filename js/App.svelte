<script>
	import ImageView from './ImageView.svelte';
	import Thumbnail from './Thumbnail.svelte';

	const baseUrl = BASE_URL;

	let rootPath = "";

	let dirList = [];
	let fileList = [];
	async function loadPage(path){
		const res = await fetch(`${baseUrl}/file_list/${path}`);
		const json = await res.json();
		dirList = json.dirs;
		fileList = json.files;
		selectedFile = null;
	}

	let selectedFile = null;

	function setFocus(evt){
		selectedFile = evt.detail;
	}

	function defocus(){
		selectedFile = null;
	}

	function selectDir(event){
		rootPath = event.detail;
		loadPage(rootPath);
	}

	function onHome(){
		rootPath = "";
		loadPage(rootPath);
	}

	function onUp(){
		const splitPath = rootPath.split("/");
		if(1 < splitPath.length){
			rootPath = splitPath.slice(0, splitPath.length - 1).join("/");
			loadPage(rootPath);
		}
		else{
			rootPath = "";
			loadPage(rootPath);
		}
	}

	window.addEventListener('load', () => loadPage(rootPath));
</script>

<div class="header">
	<div class="path" id="path">{rootPath}</div>
	<div class="iconContainer">
		<img class="icon" alt="home" id="homeButton" src={`${baseUrl}/home.png`} on:click={onHome}>
		<img class="icon" alt="up (U)" id="upButton" src={`${baseUrl}/up.png`} on:click={onUp}>
		<img class="icon" alt="previous (H)" id="leftButton" src={`${baseUrl}/left.png`}>
		<img class="icon" alt="next (K)" id="rightButton" src={`${baseUrl}/right.png`}>
	</div>
</div>

{#if selectedFile !== null}
<div class="imageContainer">
	<ImageView imagePath={`${baseUrl}/files/${selectedFile}`} on:defocus={defocus}/>
</div>
{/if}

<div class="scrollContents">
	<div class='dirContainer' id="thumbnails" style={selectedFile !== null ? 'top: 70%' : ''}>
		{#each dirList as dir (dir.path)}
			<Thumbnail {dir} {rootPath} {baseUrl} on:setFocus={selectDir}/>
		{/each}
		{#each fileList as file (file.path)}
			<Thumbnail image={file} {rootPath} {baseUrl} on:setFocus={setFocus}/>
		{/each}
	</div>
</div>


<style>
	.header {
		position: fixed;
		left: 0;
		top: 0;
		width: 100%;
		height: 3em;
		background-color: rgba(191, 191, 191, 0.75);
		z-index: 100;
	}

	.imageContainer {
		position: fixed;
		left: 0;
		top: 3em;
		width: 100%;
		height: 70%;
		z-index: 100;
	}

	.path {
		font-size: 1.5em;
	}

	.iconContainer {
		position: absolute;
		top: 0;
		right: 0;
	}

	.scrollContents {
		margin-top: 3em;
	}

	.dirContainer {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-evenly;
		align-content: space-between;
	}
</style>