<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let userName = "";
    $: message = `Do you want to delete user "${userName}"?`;

    const cancel = () => dispatch('cancel');
</script>

<div class="back" on:click|stopPropagation={cancel}>
    <div class="modal" on:click|stopPropagation={() => 0}>
        <h2>Confirm</h2>
        {message}
        <div>
            <button value="Ok" on:click|stopPropagation={() => dispatch('ok')}>Ok</button>
            <button value="Cancel" on:click|stopPropagation={cancel}>Cancel</button>
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
        z-index: 1100;
    }

    .modal {
        top: 0;
        left: 0;
        width: 80%;
        max-width: 500px;
        margin: auto;
        padding: 20px;
        background-color: #ffffff;
        text-align: center;
    }
</style>