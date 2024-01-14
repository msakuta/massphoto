<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let title = "";
    export let message = "";

    function submit() {
        dispatch('submit');
    }

    function cancel() {
        dispatch('cancel');
    }
</script>

<div class="back" on:click={cancel}>
    <div class="modal" on:click|stopPropagation={() => 0}>
        <h2>{title}</h2>
        {message}
        <div class="buttons">
            <button value="Ok" on:click={submit}>Ok</button>
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

    .buttons {
        margin: 4px;
    }
</style>