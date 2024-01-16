<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let userName = "";
    $: message = `Do you want to delete user "${userName}"?`;
</script>

<ModalFrame on:cancel zIndex="1200">
    <h2>Confirm</h2>
    {message}
    <div>
        <button value="Ok" on:click|stopPropagation={() => dispatch('ok')}>Ok</button>
        <button value="Cancel" on:click|stopPropagation={() => dispatch('cancel')}>Cancel</button>
    </div>
</ModalFrame>
