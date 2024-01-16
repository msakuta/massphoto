<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let title = "";
    export let message = "";
</script>

<ModalFrame on:cancel>
    <h2>{title}</h2>
    {message}
    <div class="buttons">
        <!-- svelte-ignore a11y-autofocus -->
        <button value="Ok" on:click={() => dispatch('submit')} autofocus>Ok</button>
        <button value="Cancel" on:click={() => dispatch('cancel')}>Cancel</button>
    </div>
</ModalFrame>

<style>
    .buttons {
        margin: 4px;
    }
</style>
