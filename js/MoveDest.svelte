<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';
    import MoveConfirm from './MoveConfirm.svelte';
    import { joinPath } from './joinPath';

    const dispatch = createEventDispatcher();

    export let baseUrl = "";
    export let rootPath = "";
    export let dirList = [];
    export let destDir = null;

    function onMoveOk() {
        dispatch('move', destDir.path);
        destDir = null;
    }

    function onMoveCancel(evt) {
        destDir = null;
        evt.stopPropagation();
    }

    function imagePath(dir) {
        return `${baseUrl}/thumbs/${joinPath(rootPath, joinPath(dir.path, dir.image_first))}`;
    }
</script>

{#if destDir !== null}
<MoveConfirm dirPath={destDir.path} on:ok={onMoveOk} on:cancel={onMoveCancel}/>
{/if}

<ModalFrame on:cancel={() => dispatch('cancel')}>
    <h2>Move destination</h2>
    <table>
        <tr><th>Image</th><th>Path</th><th>Move</th></tr>
        <tr>
            <td></td>
            <td>..</td>
            <td><button on:click={() => destDir = {path: ".."}}>Move</button></td>
        </tr>
        {#each dirList as dir (dir.path)}
        <tr>
            <td><img alt={dir.image_first} src={imagePath(dir)}></td>
            <td>{dir.path}</td>
            <td><button on:click={() => destDir = dir}>Move</button></td>
        </tr>
        {/each}
    </table>
    <div>
    <button value="Cancel" on:click={() => dispatch('cancel')}>Cancel</button>
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
</style>
