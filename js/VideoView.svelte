<script>
    import closeImage from '../assets/close.png';
    import { createEventDispatcher, tick } from 'svelte';

    const dispatch = createEventDispatcher();

    export let videoPath = "";

    let closeButton;
</script>

<div>
    <video id="videoContainer" controls width="100%" height="100%" preload="auto" src={videoPath}>
    Your browser does not support the <code>video</code> element.
    </video>
    <div class="buttonContainer">
        <img class="button barButton" bind:this={closeButton} src={closeImage} alt="Close" on:click={() => dispatch('close')}>
    </div>
</div>

<style>
    .button {
        opacity: 0.5;
    }

    .button:hover {
        opacity: 1;
    }

    .buttonContainer {
        position: absolute;
        left: 0;
        top: 64px;
    }
</style>
