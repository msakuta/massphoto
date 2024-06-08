<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let dirPath = "";
    $: message = `Do you want to move to directory "${dirPath}"?`;
</script>

<ModalFrame on:cancel zIndex="1200">
    <h2>Confirm</h2>
    {message}
    <div>
        <button value="Ok" on:click|stopPropagation={() => dispatch('ok')}>Ok</button>
        <button value="Cancel" on:click|stopPropagation={() => dispatch('cancel')}>Cancel</button>
    </div>
</ModalFrame>
