<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let message = "New User";
    let name = "";
    let password = "";
    let passwordCheck = "";

    function cancel() {
        dispatch('cancel');
    }
</script>

<div class="back" on:click={cancel}>
    <div class="modal" on:click|stopPropagation={() => 0}>
        <h2>{message}</h2>
        <label>User name:
            <input type="text" bind:value={name}>
        </label>
        <label>Password:
            <input type="password" bind:value={password}>
        </label>
        <label>Retype password:
            <input type="password" bind:value={passwordCheck}>
        </label>
        <div>
            <button value="Ok" on:click={dispatch('submit', {name, password, passwordCheck})}>Ok</button>
            <button value="Cancel" on:click={cancel}>Cancel</button>
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
</style>