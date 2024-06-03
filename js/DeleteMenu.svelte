<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();
</script>

<span class="button" on:click={() => dispatch('ok')} >
    Ok
</span>
<span class="button" on:click={() => dispatch('cancel')} >
    Cancel
</span>


<style>
    .button {
        background-color: #F0F8FF;
        padding: 5px;
        margin: 5px;
        border: solid green 4px;
    }

    .button:hover {
        background-color: #ffffff;
        border: solid #00ff00 4px;
    }
</style>
