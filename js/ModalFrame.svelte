<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let zIndex = 1100;

    const cancel = () => dispatch('cancel');
</script>

<div class="back" style="z-index: {zIndex}" on:click|stopPropagation={cancel}>
    <div class="modal" on:click|stopPropagation={() => 0}>
        <slot/>
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
        z-index: 1100;
    }

    .modal {
        display: flex;
        flex-direction: column;
        width: 80%;
        max-width: 500px;
        max-height: 80%;
        margin: auto;
        padding: 20px;
        background-color: #ffffff;
        text-align: center;
    }
</style>
