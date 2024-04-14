<script>
    import closeImage from '../assets/close.png';
    import leftAngleImage from '../assets/leftAngle.png';
    import rightAngleImage from '../assets/rightAngle.png';
    import commentButtonImage from '../assets/comment.png';
    import { createEventDispatcher, tick } from 'svelte';

    const dispatch = createEventDispatcher();

    export let videoPath = "";
    export let videoRelPath = "";
    export let descEditable = false;

    let descEdit;
    let descDiv;

    let descEditMode = false;
    let descValue = "Hello there";
    let descVisible = false;
    let closeButton;

    export let descUrl = "";
    $: getDesc(descUrl);
    async function getDesc(descUrl) {
        descEditMode = false;
        if (descUrl !== null) {
            const res = await fetch(descUrl, {
                credentials: "include",
            });
            switch (res.status) {
                case 200:
                    descValue = await res.text();
                    descVisible = true;
                    break;
                case 404:
                    descValue = "";
                    descVisible = false;
                    break;
                default:
                    descValue = "Unknown error";
                    descVisible = true;
                    break;
            }
        }
        else{
            descVisible = false;
        }
    }

    async function enterDescEditMode() {
        if(!descEditable) return;
        descEditMode = true;
        await tick();
        descEdit.focus();
    }

    async function onDescKeyDown(evt) {
        switch (evt.keyCode) {
            case 13:
                if(descEditable){
                    dispatch('setDesc', {path: videoRelPath, desc: descValue});
                    descEditMode = false;
                }
                evt.preventDefault();
                break;
            case 37: evt.stopPropagation(); break;
            case 39: evt.stopPropagation(); break;
        }
    }

    function focusout() {
        descEditMode = false;
    }

    function toggleDesc() {
        descVisible = !descVisible;
        descEditMode = false;
    }
</script>

<div>
    <video id="videoContainer" controls width="100%" height="100%" preload="auto" src={videoPath}>
    Your browser does not support the <code>video</code> element.
    </video>
    <div class="buttonContainer">
        <img class="button barButton" bind:this={closeButton} src={closeImage} alt="Close" on:click={() => dispatch('close')}>
    </div>
    <img class="button commentButton" src={commentButtonImage} alt="Description" on:click={toggleDesc}>
    <img class="button prevButton" src={leftAngleImage} alt="Prev" on:click={() => dispatch('prev', videoPath)}>
    <img class="button nextButton" src={rightAngleImage} alt="Next" on:click={() => dispatch('next', videoPath)}>
    {#if descEditMode}
        <textarea class="textPosition" bind:this={descEdit} on:keydown={onDescKeyDown} on:focusout={focusout} bind:value={descValue}></textarea>
    {:else if descVisible}
        <div class="textPosition commentShow" bind:this={descDiv} on:click={enterDescEditMode}>{descValue}</div>
    {/if}
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

    .commentButton {
        position: absolute;
        left: 0;
        bottom: 0;
    }

    .prevButton {
        position: absolute;
        left: 48px;
        top: 0;
        bottom: 0;
        margin: auto;
    }

    .nextButton {
        position: absolute;
        right: 48px;
        top: 0;
        bottom: 0;
        margin: auto;
    }

    .textPosition {
        position: absolute;
        width: 70%;
        height: 60px;
        left: 0;
        right: 0;
        bottom: 0;
        margin: auto;
    }

    .commentShow {
        background-color: rgba(255, 255, 255, 0.75);
    }
</style>
