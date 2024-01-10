<script>
	import ImageView from './ImageView.svelte';
	import VideoView from './VideoView.svelte';
	import Thumbnail from './Thumbnail.svelte';
	import PasswordEntry from './PasswordEntry.svelte';
	import ErrorMessage from './ErrorMessage.svelte';
	import { joinPath } from './joinPath';

	const baseUrl = BASE_URL;

	let rootPath = "";

	let unlockedDirs = {};

	let dirList = [];
	let fileList = [];
	async function loadPage(path){
		const headers = {};
		if(path in unlockedDirs) headers["X-Auth"] = unlockedDirs[path];
		const res = await fetch(`${baseUrl}/file_list/${path}`, {
			headers
		});
		if(!res.ok){
			// If the album is password locked, attempt unlock
			if(res.status === 403){
				showingUnlockDialog = true;
				unlockAttemptPath = path;
				return;
			}
			errorMessage = await res.text();
			return;
		}
		const json = await res.json();
		dirList = json.dirs;
		fileList = json.files;
		selectedFile = null;
		rootPath = path;
	}

	let selectedFile = null;

	let showingLockDialog = false;
	let showingUnlockDialog = false;
	let unlockAttemptPath = null;

	let errorMessage = null;

	// let sessionId = "";

	async function createOrRestoreSession() {
		// if(document.cookie){
		// 	sessionId = document.cookie;
		// 	console.log(`SessionId restored: ${sessionId}`);
		// 	return;
		// }
		const res = await fetch(`${baseUrl}/sessions`, {
			method: "POST",
			body: "",
		});
		// sessionId = await res.text();
		// document.cookie = sessionId;
	}

	function setFocus(evt){
		selectedFile = evt.detail;
	}

	function defocus(){
		selectedFile = null;
	}

	function selectDir(event){
		loadPage(event.detail);
	}

	function onHome(){
		loadPage("");
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

	function onLock() {
		showingLockDialog = true;
	}

	async function submitPassword(evt) {
		const password = evt.detail;
		console.log(`Locking ${rootPath} with printing password!! Bad boy!! ${password}`);
		const res = await fetch(`${baseUrl}/albums/${rootPath}/lock`, {
			method: "POST",
			mode: "cors",
			headers: {
				"Content-Type": "text/plain"
			},
			body: password,
		});
		const text = await res.text();
		console.log(`lock res: ${text}`);
		showingLockDialog = false;
	}

	function cancelPassword() {
		showingLockDialog = false;
	}

	function tryUnlock(evt) {
		unlockedDirs[unlockAttemptPath] = evt.detail;
		showingUnlockDialog = false;
		loadPage(unlockAttemptPath);
	}

	function cancelUnlock() {
		showingUnlockDialog = false;
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

	function onCloseErrorMessage() {
		errorMessage = null;
	}

	window.addEventListener('load', () => loadPage(rootPath));
</script>

{#if errorMessage !== null}
<ErrorMessage message={errorMessage} on:close={onCloseErrorMessage}/>
{:else if showingLockDialog}
<PasswordEntry on:submit={submitPassword} on:cancel={cancelPassword}/>
{:else if showingUnlockDialog}
<PasswordEntry message="Enter password to unlock:" on:submit={tryUnlock} on:cancel={cancelUnlock}/>
{/if}

<div class="header">
	<div class="path" id="path">{rootPath}</div>
	<div class="iconContainer">
		<img class="icon" alt="home" id="homeButton" src={`${baseUrl}/home.png`} on:click={onHome}>
		<img class="icon" alt="up (U)" id="upButton" src={`${baseUrl}/up.png`} on:click={onUp}>
		<img class="icon" alt="previous (H)" id="leftButton" src={`${baseUrl}/left.png`}>
		<img class="icon" alt="next (K)" id="rightButton" src={`${baseUrl}/right.png`}>
		<img class="icon" alt="home" src={`${baseUrl}/lock.png`} on:click={onLock}>
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
		margin-right: 20px;
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

<svelte:window on:keydown={onKeyDown} on:load={createOrRestoreSession}/>
