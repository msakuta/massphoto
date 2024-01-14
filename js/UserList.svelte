<script>
    import { createEventDispatcher } from 'svelte';
    import DeleteConfirm from './DeleteConfirm.svelte';

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

<div class="back" on:click={close}>
    {#if deletingUser !== null}
    <DeleteConfirm userName={deletingUser.name} on:ok={onDeleteOk} on:cancel={onDeleteConfirmCancel}/>
    {/if}

    <div class="modal" on:click|stopPropagation={() => 0}>
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
        <button value="Close" on:click={close}>Close</button>
        </div>
    </div>
</div>


<style>
    .back {
        position: fixed;
        left: 0;
        top: 0;
        width: 100%;
        height: 100%;
        margin: 0;
        padding-top: 100px;
        background-color: rgba(0, 0, 0, 0.75);
        z-index: 1000;
    }

    .modal {
        top: 0;
        left: 0;
        width: 80%;
        max-width: 500px;
        margin: auto;
        padding: 20px;
        background-color: #f3f3f3;
        text-align: center;
    }

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