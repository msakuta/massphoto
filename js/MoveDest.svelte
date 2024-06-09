<script>
    import { createEventDispatcher } from 'svelte';
    import upImage from '../assets/up.png';
    import ModalFrame from './ModalFrame.svelte';
    import MoveConfirm from './MoveConfirm.svelte';
    import ErrorMessage from './ErrorMessage.svelte';
    import { joinPath } from './joinPath';

    const dispatch = createEventDispatcher();

    export let baseUrl = "";
    export let rootPath = "";
    export let dirList = [];
    export let destDir = null;

    let errorMessage = null;

    function onMoveOk() {
        dispatch('move', joinPath(rootPath, destDir.path));
        destDir = null;
    }

    function onMoveCancel(evt) {
        destDir = null;
        evt.stopPropagation();
    }

    function imagePath(dir) {
        return `${baseUrl}/thumbs/${joinPath(rootPath, joinPath(dir.path, dir.image_first))}`;
    }

    function onCloseErrorMessage() {
        errorMessage = null;
    }

    async function clickDir(dirPath) {
        rootPath = joinPath(rootPath, dirPath);
        const res = await fetch(`${baseUrl}/file_list/${rootPath}`, {
            method: "GET",
            credentials: "include",
        });
        if(!res.ok){
            errorMessage = await res.text();
            return;
        }
        const json = await res.json();
        dirList = json.dirs;
    }
</script>

{#if errorMessage !== null}
<ErrorMessage message={errorMessage} on:close={onCloseErrorMessage}/>
{:else if destDir !== null}
<MoveConfirm dirPath={joinPath(rootPath, destDir.path)} on:ok={onMoveOk} on:cancel={onMoveCancel}/>
{/if}

<ModalFrame on:cancel={() => dispatch('cancel')}>
    <h2>Move destination</h2>
    <div class="margin">Current path: {rootPath}</div>
    <div class="scrollable">
    <table>
        <tr><th>Image</th><th>Path</th><th>Move</th></tr>
        <tr>
            <td></td>
            <td>. (Current directory)</td>
            <td><button on:click={() => destDir = {path: ""}}>Move</button></td>
        </tr>
        {#if rootPath !== ""}
        <tr>
            <td><img alt="Up" src={upImage} on:click={() => clickDir("..")}></td>
            <td><span on:click={() => clickDir("..")}>.. (Parent directory)</span></td>
            <td><button on:click={() => destDir = {path: ".."}}>Move</button></td>
        </tr>
        {/if}
        {#each dirList as dir (dir.path)}
        <tr>
            <td><img alt={dir.image_first} src={imagePath(dir)} on:click={() => clickDir(dir.path)}></td>
            <td><span on:click={() => clickDir(dir.path)}>{dir.path}</span></td>
            <td><button on:click={() => destDir = dir}>Move</button></td>
        </tr>
        {/each}
    </table>
    </div>
    <div>
    <button class="margin" value="Cancel" on:click={() => dispatch('cancel')}>Cancel</button>
    </div>
</ModalFrame>

<style>
    table {
        table-layout: auto;
        width: 80%;
        margin-left: auto;
        margin-right: auto;
        border: 1px solid;
        border-collapse: collapse;
    }
    
    td, th {
        border: 1px solid #7f7f7f;
        padding: 4px;
    }

    .margin {
        margin: 4px;
    }

    .scrollable {
        max-height: 80%;
        overflow-y: scroll;
    }
</style>
