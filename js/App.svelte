<script>
	import Nested from './Nested.svelte';

	const host = "localhost:10008";

	let rootPath = "";

	let dirList = [];
	let fileList = [];
	async function getFileList(path){
		const res = await fetch(`http://${host}/file_list/${path}`);
		const json = await res.json();
		dirList = json.dirs;
		fileList = json.files;
	}

	window.addEventListener('load', () => getFileList(rootPath));
</script>

<p>These styles...</p>
<Nested/>

<ul>
{#each dirList as dir}
<li>{dir.image_first}: {dir.file_count}</li>
{/each}
{#each fileList as file}
<li>{file.basename}: {file.label}</li>
{/each}
</ul>

<style>
	p {
		color: purple;
		font-family: 'Times New Roman', cursive;
		font-size: 2em;
	}
</style>