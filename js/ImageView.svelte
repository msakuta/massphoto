<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let imagePath = "";

    function mouseup(event) {
        event.preventDefault();
        if (event.button === 0) {
            dispatch('defocus');
        }
        return false;
    }

    function contextmenu(event) {
        event.preventDefault();
    }
</script>

<div>
    <img class="zoomInt" src={imagePath}
        on:mouseup={mouseup} on:contextmenu={contextmenu} alt={imagePath}>
</div>

<style>
    .zoomInt {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }
</style>