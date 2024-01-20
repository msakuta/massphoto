<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let message = "Upload a File";
    export let baseUrl;
    let action = `${baseUrl}/upload`;
    let fileInput;

    function submit(evt) {
        // dispatch('submit', {name, password});
        const form = evt.currentTarget;
        const url = form.action;
        const file = fileInput.files[0];
        console.log(`form: ${form}, file: ${file}`);
        const formData = new FormData();
        formData.append("file", file);
        fetch(url, {
            method: 'POST',
            body: formData,
        });
        evt.preventDefault();
    }
</script>

<ModalFrame on:cancel>
    <h2>{message}</h2>
    <form action={action} method="post" enctype="multipart/form-data" on:submit={submit}>
        <label>File
            <input bind:this={fileInput} name="file" type="file">
        </label>
        <div>
            <button value="Ok" type="submit">Ok</button>
            <button value="Cancel" on:click={() => dispatch('cancel')}>Cancel</button>
        </div>
    </form>
</ModalFrame>
