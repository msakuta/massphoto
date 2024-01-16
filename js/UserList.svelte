<script>
    import { createEventDispatcher } from 'svelte';
    import DeleteConfirm from './DeleteConfirm.svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let users = [];

    function close() { dispatch('close') }

    let deletingUser = null;

    function onDeleteOk() {
        dispatch('delete', deletingUser.id);
        deletingUser = null;
    }

    function onDeleteConfirmCancel(evt) {
        deletingUser = null;
        evt.stopPropagation();
    }
</script>

{#if deletingUser !== null}
<DeleteConfirm userName={deletingUser.name} on:ok={onDeleteOk} on:cancel={onDeleteConfirmCancel}/>
{/if}

<ModalFrame on:cancel={() => dispatch('close')}>
    <h2>User List</h2>
    <table>
        <tr><th>Id</th><th>Name</th><th>Password</th><th>Delete</th></tr>
        {#each users as user (user.id)}
        <tr>
            <td>{user.id}</td>
            <td>{user.name}</td>
            <td>{user.password}</td>
            <td><button on:click={() => deletingUser = user}>Delete</button></td>
        </tr>
        {/each}
    </table>
    <div>
    <button value="Close" on:click={() => dispatch('close')}>Close</button>
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
