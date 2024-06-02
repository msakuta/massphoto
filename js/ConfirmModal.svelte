<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let title = "";
    export let message = "";
    export let cancelButton = true;
</script>

<ModalFrame on:cancel>
    <h2>{title}</h2>
    {message}
    <div class="buttons">
        <!-- svelte-ignore a11y-autofocus -->
        <button value="Ok" on:click={() => dispatch('submit')} autofocus>Ok</button>
        {#if cancelButton}
            <button value="Cancel" on:click={() => dispatch('cancel')}>Cancel</button>
        {/if}
    </div>
</ModalFrame>

<style>
    .buttons {
        margin: 4px;
    }
</style>
