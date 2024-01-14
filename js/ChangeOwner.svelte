<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let users = [];

    function close() { dispatch('close') }

    function onSetOwner(user) {
        if(user === null) return;
        dispatch('ok', user.id);
    }
</script>

<div class="back" on:click={close}>
    <div class="modal" on:click|stopPropagation={() => 0}>
        <h2>Choose a new owner of this album</h2>
        <table>
            <tr><th>Id</th><th>Name</th><th>Password</th><th>Set owner</th></tr>
            {#each users as user (user.id)}
            <tr>
                <td>{user.id}</td>
                <td>{user.name}</td>
                <td>{user.password}</td>
                <td><button on:click={() => onSetOwner(user)}>Set owner</td>
            </tr>
            {/each}
        </table>
        <div>
            <button value="Cancel" on:click={close}>Cancel</button>
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