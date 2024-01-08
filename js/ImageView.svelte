<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let imagePath = "";
    let scale = 1.;
    let translate = [0, 0];
    $: imageTransform = `translate(${translate[0]}px, ${translate[1]}px) scale(${scale})`;
    let client;

    export let magnifyPath = "";
    let magnifyButton;
    export let minifyPath = "";
    let minifyButton;

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

    let dragStart = null;
    let dragMoved = false;
    function mousedown(event) {
        if(event.target === magnifyButton || event.target === minifyButton) return;
        event.preventDefault();
        dragStart = [event.clientX, event.clientY];
    }

    function mouseup(event) {
        if(event.target === magnifyButton || event.target === minifyButton) return;
        if(!dragMoved && event.button === 0){
            dispatch('defocus');
            event.preventDefault();
        }
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

    function magnify(event) {
        event.preventDefault();
        scale *= 1.2;
    }

    function minify(event) {
        event.preventDefault();
        scale /= 1.2;
    }
</script>

<div class="container" bind:this={client} on:wheel={applyZoom}
        on:mouseup={mouseup} on:contextmenu={contextmenu} on:mousedown={mousedown} on:mousemove={mousemove} on:mouseleave={mouseleave}>
    <img style="transform: {imageTransform}" class="zoomInt noPointer" src={imagePath} alt={imagePath} on:load={getImageSize}>
    <div class="buttonContainer">
        <img class="button" bind:this={magnifyButton} src={magnifyPath} alt={magnifyPath} on:click={magnify}>
        <img class="button" style="top: 48px" bind:this={minifyButton} src={minifyPath} alt={minifyPath} on:click={minify}>
    </div>
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
        position: absolute;
        left: 0;
        top: 0;
    }
</style>
