<script>
	import homeImage from '../assets/home.png';
	import upImage from '../assets/up.png';
	import leftImage from '../assets/left.png';
	import rightImage from '../assets/right.png';
	import hamburgerImage from '../assets/hamburger.png';

	import ImageView from './ImageView.svelte';
	import VideoView from './VideoView.svelte';
	import Thumbnail from './Thumbnail.svelte';
	import PasswordEntry from './PasswordEntry.svelte';
	import UserLogin from './UserLogin.svelte';
	import ConfirmModal from './ConfirmModal.svelte';
	import UserAdd from './UserAdd.svelte';
	import UserList from './UserList.svelte';
	import ChangePassword from './ChangePassword.svelte';
	import ChangeOwner from './ChangeOwner.svelte';
	import TitleBarButton from './TitleBarButton.svelte';
	import MainMenu from './MainMenu.svelte';
	import ErrorMessage from './ErrorMessage.svelte';
	import { joinPath } from './joinPath';

	const baseUrl = BASE_URL;

	let rootPath = "";

	let dirList = [];
	let fileList = [];
	async function loadPage(path){
		const headers = {  };
		const res = await fetch(`${baseUrl}/file_list/${path}`, {
			headers,
			credentials: "include",
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
	let userName = "";
	let userIsAdmin = false;

	async function createOrRestoreSession() {
		const res = await fetch(`${baseUrl}/sessions`, {
			method: "GET",
			credentials: "include",
		});
		if(!res.ok) errorMessage = await res.text();
	}

	async function getUserStatus() {
		const res = await fetch(`${baseUrl}/user_status`, {
			credentials: "include",
		});
		if(!res.ok){
			errorMessage = await res.text();
			return;
		}
		let result = await res.json();
		userName = result.logged_in ? result.name : "";
		userIsAdmin = result.is_admin;
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

	let showingUserLoginDialog = false;

	function onStartLogin() {
		showingMainMenu = false;
		showingUserLoginDialog = true;
	}

	async function onUserLogin(evt) {
		const res = await fetch(`${baseUrl}/users/login`, {
			method: "POST",
			credentials: "include",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				name: evt.detail.name,
				password: evt.detail.password,
			}),
		});
		if (!res.ok) {
			const response = await res.text();
			errorMessage = `User login failed: ${response}`;
			return;
		}
		location.reload();
	}

	async function onCancelUserLogin() {
		showingUserLoginDialog = false;
	}

	let showingUserLogoutDialog = false;

	function onStartLogout() {
		showingUserLogoutDialog = true;
	}

	async function onUserLogout() {
		const res = await fetch(`${baseUrl}/user_logout`, {
			method: "POST",
			credentials: "include",
		});
		if (!res.ok) {
			const response = await res.text();
			errorMessage = `User logout failed: ${response}`;
			return;
		}
		location.reload();
	}

	function onCancelUserLogout() {
		showingUserLogoutDialog = false;
	}

	let showingUserAddDialog = false;

	function onStartUserAdd() {
		showingUserAddDialog = true;
	}

	async function onUserAdd(evt) {
		if(evt.detail.password !== evt.detail.passwordCheck){
			errorMessage = "The retyped password does not match. Try again";
			return;
		}
		const res = await fetch(`${baseUrl}/users`, {
			method: "POST",
			credentials: "include",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				name: evt.detail.name,
				password: evt.detail.password
			})
		});
		if (!res.ok) {
			const response = await res.text();
			errorMessage = `User add failed: ${response}`;
			return;
		}
		showingUserAddDialog = false;
	}

	function onCancelUserAdd() {
		showingUserAddDialog = false;
	}

	let showingChangePasswordDialog = false;

	function onStartChangePassword() {
		showingChangePasswordDialog = true;
	}

	async function onChangePassword(evt) {
		if(evt.detail.password !== evt.detail.passwordCheck){
			errorMessage = "The retyped password does not match. Try again";
			return;
		}
		const res = await fetch(`${baseUrl}/set_password`, {
			method: "POST",
			credentials: "include",
			body: evt.detail.password,
		});
		if (!res.ok) {
			const response = await res.text();
			errorMessage = `Change password failed: ${response}`;
			return;
		}
		showingUserAddDialog = false;
	}

	function cancelChangePassword() {
		showingChangePasswordDialog = false;
	}

	function onLock() {
		showingLockDialog = true;
	}

	async function lockWithPassword(evt) {
		const password = evt.detail;
		const res = await fetch(`${baseUrl}/albums/${rootPath}/lock`, {
			method: "POST",
			mode: "cors",
			headers: {
				"Content-Type": "text/plain"
			},
			credentials: "include",
			body: password,
		});
		if(!res.ok){
			errorMessage = await res.text();
			return;
		}
		const text = await res.text();
		console.log(`lock res: ${text}`);
		showingLockDialog = false;
	}

	function cancelPassword() {
		showingLockDialog = false;
	}

	async function tryUnlock(evt) {
		const res = await fetch(`${baseUrl}/albums/${unlockAttemptPath}/auth`, {
			method: "POST",
			credentials: "include",
			body: evt.detail,
		});
		if(res.ok){
			const ok = await res.text();
			showingUnlockDialog = false;
			loadPage(unlockAttemptPath);
		}
		else{
			errorMessage = await res.text();
		}
	}

	async function updateUserList() {
		let res = await fetch(`${baseUrl}/users`, {
			credentials: "include",
		});
		if(!res.ok){
			errorMessage = await res.text();
			return;
		}
		users = await res.json();
	}

	function cancelUnlock() {
		showingUnlockDialog = false;
	}

	let showingUserList = false;
	let users = [];

	async function onUserList() {
		await updateUserList();
		showingUserList = true;
	}

	function onUserListClose() {
		showingUserList = false;
	}

	async function onUserDelete(evt) {
		const deletingId = evt.detail;
		const res = await fetch(`${baseUrl}/users/${deletingId}`, {
			method: "DELETE",
			credentials: "include",
		});
		if(!res.ok){
			errorMessage = await res.text();
			return;
		}
		onUserList();
	}

	let currentOwner = 1;

	let showingChangeOwnerDialog = false;

	async function onStartOwnerChange() {
		const ownerFut = (async () => {
			const res = await fetch(`${baseUrl}/albums/${rootPath}/owner`, {
				credentials: "include"
			});
			if(!res.ok){
				errorMessage = await res.text();
				return;
			}
			return parseInt(await res.text());
		})();
		const usersFut = updateUserList();
		currentOwner = (await Promise.all([ownerFut, usersFut]))[0];
		console.log(`currentOwner: ${currentOwner}`);
		showingChangeOwnerDialog = true;
	}

	async function onSetOwner(evt) {
		const res = await fetch(`${baseUrl}/albums/${rootPath}/set_owner`, {
			method: "POST",
			credentials: "include",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				user_id: evt.detail,
			}),
		});
		if(!res.ok){
			errorMessage = await res.text();
			return;
		}
		showingChangeOwnerDialog = false;
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

	let showingMainMenu = false;

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

	let showingClearCacheDialog = false;

	async function onClearCache() {
		const res = await fetch(`${baseUrl}/clear_cache`, {
			credentials: "include",
		});
		if(!res.ok){
			errorMessage = await res.text();
		}
		showingClearCacheDialog = false;
	}

	async function initialize() {
		// Get the session before fetching the first file list.
		await createOrRestoreSession();
		getUserStatus();
		loadPage(rootPath);
	}
</script>

{#if errorMessage !== null}
<ErrorMessage message={errorMessage} on:close={onCloseErrorMessage}/>
{:else if showingUserLoginDialog}
<UserLogin on:submit={onUserLogin} on:cancel={onCancelUserLogin}/>
{:else if showingUserLogoutDialog}
<ConfirmModal title="Logging Out" message="Ok to logout?" on:submit={onUserLogout} on:cancel={onCancelUserLogout}/>
{:else if showingUserAddDialog}
<UserAdd on:submit={onUserAdd} on:cancel={onCancelUserAdd}/>
{:else if showingChangePasswordDialog}
<ChangePassword on:submit={onChangePassword} on:cancel={cancelChangePassword}/>
{:else if showingLockDialog}
<PasswordEntry title="Locking Album" on:submit={lockWithPassword} on:cancel={cancelPassword}/>
{:else if showingUnlockDialog}
<PasswordEntry title="Unlocking Album" message="Enter password to unlock:" on:submit={tryUnlock} on:cancel={cancelUnlock}/>
{:else if showingClearCacheDialog}
<ConfirmModal title="Clear Cache" message="Ok to clear thumbnail cache?" on:submit={onClearCache} on:cancel={() => showingClearCacheDialog = false}/>
{/if}

{#if showingUserList}
<UserList {users} on:close={onUserListClose} on:delete={onUserDelete}/>
{:else if showingChangeOwnerDialog}
<ChangeOwner {users} {currentOwner} on:close={() => showingChangeOwnerDialog = false} on:ok={onSetOwner}/>
{/if}

{#if showingMainMenu}
<MainMenu {userName} {userIsAdmin}
	on:close={() => showingMainMenu = false}
	on:login={onStartLogin}
	on:logout={onStartLogout}
	on:userAdd={onStartUserAdd}
	on:userList={onUserList}
	on:changePassword={onStartChangePassword}
	on:clearCache={() => showingClearCacheDialog = true}
	on:lock={onLock}
	on:ownerChange={onStartOwnerChange} />
{/if}

<div class="header">
	<div class="path" id="path">{rootPath}</div>
	<div class="iconContainer">
		<span class="userName">{userName}</span>
		<TitleBarButton alt="home" src={homeImage} on:click={onHome} />
		<TitleBarButton alt="up (U)" src={upImage} on:click={onUp} />
		<TitleBarButton alt="previous (H)" src={leftImage} />
		<TitleBarButton alt="next (K)" src={rightImage} />
		<TitleBarButton alt="menu" src={hamburgerImage} on:click={() => showingMainMenu = true} />
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
		height: 48px;
		margin-right: 20px;
		display: flex;
	}

	.userName {
		margin: auto;
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

<svelte:window on:keydown={onKeyDown} on:load={initialize}/>
