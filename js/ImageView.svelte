<script>
    import { createEventDispatcher, tick } from 'svelte';

    const dispatch = createEventDispatcher();

    export let imagePath = "";
    let scale = 1.;
    let translate = [0, 0];
    $: imageTransform = `translate(${translate[0]}px, ${translate[1]}px) scale(${scale})`;
    let client;

    export let imageRelPath = "";

    export let buttonImageBasePath = "";
    let closePath = `${buttonImageBasePath}/close.png`;
    let closeButton;
    let magnifyPath = `${buttonImageBasePath}/magnify.png`;
    let magnifyButton;
    let minifyPath = `${buttonImageBasePath}/minify.png`;
    let minifyButton;
    let fitPath = `${buttonImageBasePath}/fit.png`;
    let fitButton;
    let leftAnglePath = `${buttonImageBasePath}/leftAngle.png`;
    let prevButton;
    let rightAnglePath = `${buttonImageBasePath}/rightAngle.png`;
    let nextButton;
    let commentButtonPath = `${buttonImageBasePath}/comment.png`;
    let commentButton;

    let commentDiv;
    let commentEdit;

    let commentEditMode = false;
    let commentValue = "Hello there";
    let commentVisible = false;

    export let commentUrl = "";
    $: getComment(commentUrl);
    async function getComment(commentUrl) {
        commentEditMode = false;
        if (commentUrl !== null) {
            const res = await fetch(commentUrl);
            switch (res.status) {
                case 200:
                    commentValue = await res.text();
                    commentVisible = true;
                    break;
                case 404:
                    commentValue = "";
                    commentVisible = false;
                    break;
                default:
                    commentValue = "Unknown error";
                    commentVisible = true;
                    break;
            }
        }
        else{
            commentVisible = false;
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

    let allButtons = [closeButton, magnifyButton, minifyButton, fitButton, commentDiv, commentEdit];

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

    async function enterCommentEditMode() {
        commentEditMode = true;
        await tick();
        commentEdit.focus();
    }

    async function onCommentKeyDown(evt) {
        switch (evt.keyCode) {
            case 13:
                dispatch('setComment', {path: imageRelPath, comment: commentValue});
                commentEditMode = false;
                evt.preventDefault();
                break;
            case 37: evt.stopPropagation(); break;
            case 39: evt.stopPropagation(); break;
        }
    }

    function focusout() {
        commentEditMode = false;
    }

    function toggleComment() {
        commentVisible = !commentVisible;
        commentEditMode = false;
    }
</script>

<div class="container" bind:this={client} on:wheel={applyZoom}
        on:mouseup={mouseup} on:contextmenu={contextmenu} on:mousedown={mousedown} on:mousemove={mousemove} on:mouseleave={mouseleave}>
    <img style="transform: {imageTransform}" class="zoomInt noPointer" src={imagePath} alt={imagePath} on:load={getImageSize}>
    <div class="buttonContainer">
        <img class="button barButton" bind:this={closeButton} src={closePath} alt="Close" on:click={close}>
        <img class="button barButton" style="top: 48px" bind:this={magnifyButton} src={magnifyPath} alt="Magnify" on:click={magnify}>
        <img class="button barButton" style="top: 96px" bind:this={minifyButton} src={minifyPath} alt="Minify" on:click={minify}>
        <img class="button barButton" style="top: 144px" bind:this={fitButton} src={fitPath} alt="Fit" on:click={fit}>
    </div>
    <img class="button commentButton" bind:this={commentButton} src={commentButtonPath} alt="Comment" on:click={toggleComment}>
    <img class="button prevButton" bind:this={prevButton} src={leftAnglePath} alt="Prev" on:click={dispatch('prev', imagePath)}>
    <img class="button nextButton" bind:this={nextButton} src={rightAnglePath} alt="Next" on:click={next}>
    {#if commentEditMode}
        <textarea class="textPosition" bind:this={commentEdit} on:keydown={onCommentKeyDown} on:focusout={focusout} bind:value={commentValue}></textarea>
    {:else if commentVisible}
        <div class="textPosition commentShow" bind:this={commentDiv} on:click={enterCommentEditMode}>{commentValue}</div>
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
