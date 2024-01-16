<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let title = "";
    export let message = "Enter password (empty to remove protection):";
    let password = "";

    const submit = () => dispatch('submit', password);

    function onKeyDown(evt) {
        switch(evt.keyCode) {
            case 13: evt.preventDefault(); submit(); break;
        }
    }
</script>

<ModalFrame on:cancel>
    <h2>{title}</h2>
    {message}
    <!-- svelte-ignore a11y-autofocus -->
    <input type="password" bind:value={password} autofocus on:keydown={onKeyDown}>
    <div>
    <button value="Ok" on:click={submit}>Ok</button>
    <button value="Cancel" on:click={() => dispatch('cancel')}>Cancel</button>
    </div>
</ModalFrame>
