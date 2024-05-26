<script>
    import closeImage from '../assets/close.png';
    import magnifyImage from '../assets/magnify.png';
    import minifyImage from '../assets/minify.png';
    import fitImage from '../assets/fit.png';
    import leftAngleImage from '../assets/leftAngle.png';
    import rightAngleImage from '../assets/rightAngle.png';
    import commentButtonImage from '../assets/comment.png';
    import { createEventDispatcher, tick } from 'svelte';

    const dispatch = createEventDispatcher();

    export let imagePath = "";
    let scale = 1.;
    let translate = [0, 0];
    $: imageTransform = `translate(${translate[0]}px, ${translate[1]}px) scale(${scale})`;
    let client;

    export let imageRelPath = "";
    export let descEditable = false;

    let closeButton;
    let magnifyButton;
    let minifyButton;
    let fitButton;
    let descDiv;
    let descEdit;

    let descEditMode = false;
    let descValue = "Hello there";
    let descVisible = false;

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

    function applyZoom(event){
        if(focus === null) return true;
        event.preventDefault();

        var deltaScale = Math.exp(event.deltaY < 0 ? 0.1 : -0.1);

        var x = event.clientX;
        var y = event.clientY;
        // translate[0] = -x * deltaScale + x;
        // translate[1] = -y * deltaScale + y;

        scale *= deltaScale;

        // Restrict scale
        scale = Math.min(Math.max(0.1, scale), 20);
    }

    let allButtons = [closeButton, magnifyButton, minifyButton, fitButton, descDiv, descEdit];

    let dragStart = null;
    let dragMoved = false;
    function mousedown(event) {
        if(allButtons.map(button => event.target === button).reduce((acc, cur) => acc || cur)) return;
        event.preventDefault();
        dragStart = [event.clientX, event.clientY];
    }

    function mouseup(event) {
        if(allButtons.map(button => event.target === button).reduce((acc, cur) => acc || cur)) return;
        dragStart = null;
        dragMoved = false;
    }

    function contextmenu(event) {
        event.preventDefault();
    }

    function mousemove(event) {
        if(dragStart && focus){
            translate[0] += event.clientX - dragStart[0];
            translate[1] += event.clientY - dragStart[1];
            dragStart = [event.clientX, event.clientY];
            dragMoved = true;
        }
    }

    function mouseleave() {
        dragStart = null;
        dragMoved = false;
    }

    function getImageSize(event) {
        let width = event.target.width;
        let height = event.target.height;
        scale = Math.min(client.clientWidth / width, client.clientHeight / height);
    }

    function close(event) {
        dispatch('defocus');
    }

    function magnify(event) {
        event.preventDefault();
        scale *= 1.2;
    }

    function minify(event) {
        event.preventDefault();
        scale /= 1.2;
    }

    function fit(event) {
        event.preventDefault();
        scale = 1;
        translate = [0, 0];
    }

    function next() {
        dispatch('next', imagePath);
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
                    dispatch('setDesc', {path: imageRelPath, desc: descValue});
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

<div class="container" bind:this={client} on:wheel={applyZoom}
        on:mouseup={mouseup} on:contextmenu={contextmenu} on:mousedown={mousedown} on:mousemove={mousemove} on:mouseleave={mouseleave}>
    <img style="transform: {imageTransform}" class="zoomInt noPointer" src={imagePath} alt={imagePath} on:load={getImageSize}>
    <div class="buttonContainer">
        <img class="button barButton" bind:this={closeButton} src={closeImage} alt="Close" on:click={close}>
        <img class="button barButton" style="top: 48px" bind:this={magnifyButton} src={magnifyImage} alt="Magnify" on:click={magnify}>
        <img class="button barButton" style="top: 96px" bind:this={minifyButton} src={minifyImage} alt="Minify" on:click={minify}>
        <img class="button barButton" style="top: 144px" bind:this={fitButton} src={fitImage} alt="Fit" on:click={fit}>
    </div>
    <img class="button commentButton" src={commentButtonImage} alt="Description" on:click={toggleDesc}>
    <img class="button prevButton" src={leftAngleImage} alt="Prev" on:click={dispatch('prev', imagePath)}>
    <img class="button nextButton" src={rightAngleImage} alt="Next" on:click={next}>
    {#if descEditMode}
        <textarea class="textPosition" bind:this={descEdit} on:keydown={onDescKeyDown} on:focusout={focusout} bind:value={descValue}></textarea>
    {:else if descVisible}
        <div class="textPosition commentShow" bind:this={descDiv} on:click={enterDescEditMode}>{descValue}</div>
    {/if}
</div>

<style>
    .container {
        width: 100%;
        height: 100%;
    }

    .zoomInt {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }
    .noPointer {
        pointer-events: none;
    }
    .buttonContainer {
        position: absolute;
        left: 0;
        top: 64px;
    }

    .button {
        opacity: 0.5;
    }

    .button:hover {
        opacity: 1;
    }

    .barButton {
        position: absolute;
        left: 0;
        top: 0;
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
