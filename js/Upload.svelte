<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let message = "Upload";
    let files;

    function submit() {
        dispatch('submit', {files});
    }

    function cancel() {
        dispatch('cancel');
    }
</script>

<ModalFrame on:cancel>
    <h2>{message}</h2>
    <!-- <form action="/upload" method> -->
        <label>File to upload:
            <input accept="image/png, image/jpeg" bind:files enctype="multipart/form-data" name="file" type="file">
        </label>
        <div>
            <button value="Ok" on:click={submit}>Ok</button>
            <button value="Cancel" on:click={cancel}>Cancel</button>
        </div>
    <!-- </form> -->
</ModalFrame>
