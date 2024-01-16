<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let message = "User Login";
    let name = "";
    let password = "";

    function submit() {
        dispatch('submit', {name, password});
    }

    function onKeyDown(evt) {
        switch(evt.keyCode) {
            case 13: evt.preventDefault(); submit(); break;
        }
    }
</script>

<ModalFrame on:cancel>
    <h2>{message}</h2>
    <label>User name:
        <!-- svelte-ignore a11y-autofocus -->
        <input type="text" bind:value={name} autofocus>
    </label>
    <label>Password:
        <input type="password" bind:value={password} on:keydown={onKeyDown}>
    </label>
    <div>
        <button value="Ok" on:click={submit}>Ok</button>
        <button value="Cancel" on:click={() => dispatch('cancel')}>Cancel</button>
    </div>
</ModalFrame>
