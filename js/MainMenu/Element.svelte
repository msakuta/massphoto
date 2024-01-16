<script>
    import { createEventDispatcher } from 'svelte';
    const dispatch = createEventDispatcher();
    export let src = "";
    export let label = "";
    export let alt = "";
</script>


<div class="noselect row" on:click={() => dispatch('click')}>
    <img class="icon" draggable="false" {alt} src={src}>
    <span class="valign">{label}</span>
</div>

<style>
    .row {
        display: flex;
        margin: 1px;
    }

    .row:hover {
        border: 1px solid red;
        background-color: #ffffaf;
        margin: 0px;
    }

    .valign {
        width: 100%;
        padding: 4px;
        margin: auto;
    }
</style>
