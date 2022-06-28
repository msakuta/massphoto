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

<p>These styles...</p>

<ul>
</ul>

<div class='dirContainer' id="thumbnails">
	{#each dirList as dir}
		<Thumbnail {dir} {rootPath} {baseUrl}/>
	{/each}
	{#each fileList as file}
		<Thumbnail image={file} {rootPath} {baseUrl}/>
	{/each}
</div>



<style>
	p {
		color: purple;
		font-family: 'Times New Roman', cursive;
		font-size: 2em;
	}
	.dirContainer {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-evenly;
		align-content: space-between;
	}
</style>