<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let users = [];
    export let currentOwner = null;

    $: userList = users.map(user => ({
        id: user.id,
        name: user.name,
        password: user.password,
        class_: rowClass(user),
    }));

    function onSetOwner(user) {
        if(user === null) return;
        dispatch('ok', user.id);
    }

    function rowClass(user) {
        return currentOwner === user.id ? "currentOwner" : "";
    }
</script>

<ModalFrame on:cancel>
    <h2>Choose a new owner of this album</h2>
    <table>
        <tr><th>Id</th><th>Name</th><th>Password</th><th>Set owner</th></tr>
        {#each userList as user (user.id)}
        <tr class={user.class_}>
            <td>{user.id}</td>
            <td>{user.name}</td>
            <td>{user.password}</td>
            <td><button on:click={() => onSetOwner(user)}>Set owner</td>
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

    .currentOwner {
        font-weight: bold;
    }
</style>
