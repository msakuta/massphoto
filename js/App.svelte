<script>
	import Thumbnail from './Thumbnail.svelte';

	const host = "localhost:10008";
	const baseUrl = `http://${host}`;

	let rootPath = "";

	let dirList = [];
	let fileList = [];
	async function getFileList(path){
		const res = await fetch(`${baseUrl}/file_list/${path}`);
		const json = await res.json();
		dirList = json.dirs;
		fileList = json.files;
	}

	window.addEventListener('load', () => getFileList(rootPath));
</script>

<div class="header">
	<div class="path" id="path">{rootPath}</div>
	<div class="iconContainer">
		<img class="icon" alt="home" id="homeButton" src={`${baseUrl}/home.png`}>
		<img class="icon" alt="up (U)" id="upButton" src={`${baseUrl}/up.png`}>
		<img class="icon" alt="previous (H)" id="leftButton" src={`${baseUrl}/left.png`}>
		<img class="icon" alt="next (K)" id="rightButton" src={`${baseUrl}/right.png`}>
	</div>
</div>

<div class="scrollContents">
	<div class='dirContainer' id="thumbnails">
		{#each dirList as dir}
			<Thumbnail {dir} {rootPath} {baseUrl}/>
		{/each}
		{#each fileList as file}
			<Thumbnail image={file} {rootPath} {baseUrl}/>
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