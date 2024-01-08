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

	function onPrevImage() {
		const found = fileList.map((file, idx) => [file, idx]).find(([file, _]) => joinPath(rootPath, file.path) === selectedFile);
		selectedFile = joinPath(rootPath, fileList[Math.max(0, found[1] - 1)].path);
	}

	function onNextImage() {
		const found = fileList.map((file, idx) => [file, idx]).find(([file, _]) => joinPath(rootPath, file.path) === selectedFile);
		selectedFile = joinPath(rootPath, fileList[Math.min(fileList.length - 1, found[1] + 1)].path);
	}

	let isSelectedVideo = false;

	$: {
		const found = fileList.find(file => joinPath(rootPath, file.path) === selectedFile);
		isSelectedVideo = found && found.video;
	}

	function onKeyDown(evt) {
		switch(evt.keyCode) {
			case 37: evt.preventDefault(); onPrevImage(); break;
			case 39: evt.preventDefault(); onNextImage(); break;
		}
	}

	async function onSetComment(evt) {
		const res = await fetch(`${baseUrl}/comments/${evt.detail.path}`, {
			method: "POST",
			mode: "cors",
			headers: {
				"Content-Type": "text/plain"
			},
			body: evt.detail.comment,
		});
		const text = await res.text();
		console.log(`setComment res: ${text}`);
	}

	$: commentUrl = `${baseUrl}/comments/${selectedFile}`;

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
		<ImageView imagePath={`${baseUrl}/files/${selectedFile}`}
			imageRelPath={selectedFile}
			{commentUrl}
			buttonImageBasePath={`${baseUrl}`}
			on:defocus={defocus}
			on:prev={onPrevImage}
			on:next={onNextImage}
			on:setComment={onSetComment}/>
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
		padding: 0.5em;
		left: 0;
		top: 0;
		width: 100%;
		height: 2em;
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

<svelte:window on:keydown={onKeyDown} />
