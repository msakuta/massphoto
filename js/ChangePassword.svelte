<script>
    import { createEventDispatcher } from 'svelte';
    import ModalFrame from './ModalFrame.svelte';

    const dispatch = createEventDispatcher();

    export let message = "Change Password";
    let password = "";
    let passwordCheck = "";

    function submit() {
        dispatch('submit', {password, passwordCheck});
    }

    function cancel() {
        dispatch('cancel');
    }

    function onKeyDown(evt) {
        switch(evt.keyCode) {
            case 13: evt.preventDefault(); submit(); break;
        }
    }
</script>

<ModalFrame on:cancel>
    <h2>{message}</h2>
    <label>New password:
        <!-- svelte-ignore a11y-autofocus -->
        <input type="password" bind:value={password} autofocus>
    </label>
    <label>Retype password:
        <input type="password" bind:value={passwordCheck} on:keydown={onKeyDown}>
    </label>
    <div>
        <button value="Ok" on:click={submit}>Ok</button>
        <button value="Cancel" on:click={cancel}>Cancel</button>
    </div>
</ModalFrame>
