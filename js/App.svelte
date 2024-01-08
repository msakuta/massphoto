<script>
	import ImageView from './ImageView.svelte';
	import VideoView from './VideoView.svelte';
	import Thumbnail from './Thumbnail.svelte';
	import { joinPath } from './joinPath';

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

	let isSelectedVideo = false;

	$: {
		const found = fileList.find(file => joinPath(rootPath, file.path) === selectedFile);
		isSelectedVideo = found && found.video;
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
	{#if isSelectedVideo}
		<VideoView videoPath={`${baseUrl}/files/${selectedFile}`}/>
	{:else}
		<ImageView imagePath={`${baseUrl}/files/${selectedFile}`} on:defocus={defocus}/>
	{/if}
</div>
{/if}

<div class="scrollContents" style={selectedFile !== null ? 'top: 70%' : ''}>
	<div class='dirContainer' id="thumbnails">
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
		z-index: 110;
	}

	.imageContainer {
		position: fixed;
		left: 0;
		top: 0;
		width: 100%;
		height: 70%;
		z-index: 100;
		overflow:hidden;
		background-color: #afafaf;
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
		position: relative;
		width: 75%;
		margin-top: 3em;
		margin-left: auto;
		margin-right: auto;
	}

	.dirContainer {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-evenly;
		align-content: space-between;
	}
</style>