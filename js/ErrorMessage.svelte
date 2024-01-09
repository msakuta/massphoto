<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let message = "";

    function close() { dispatch('close') }
</script>

<div class="back" on:click={close}>
    <div class="modal" on:click|stopPropagation={() => 0}>
        {message}
        <div>
        <button value="Ok" on:click={close}>Ok</button>
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
        background-color: #ffa3a3;
        text-align: center;
    }
</style>