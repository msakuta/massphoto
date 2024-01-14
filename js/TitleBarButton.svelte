<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    export let src = "";
    export let alt = "";

    let element;
    let hovering = false;
    let tooltipLeft = 0;
    let tooltipTop = 0;

    function startHovering() {
        hovering = true;
        const rect = element.getBoundingClientRect();
        tooltipLeft = rect.left;
        tooltipTop = rect.bottom;
    }

    $: tooltipStyle = `left: ${tooltipLeft}px; top: ${tooltipTop}px`;
</script>

<span on:mouseenter={startHovering} on:mouseleave={() => hovering = false}>
    <img class="icon" bind:this={element} {alt} {src} on:click={() => dispatch('click')}>
</span>

{#if hovering}
<div class="tooltip" style={tooltipStyle}>
    {alt}
</div>
{/if}

<style>
    span {
        margin: 1px;
    }
    span:hover {
        border: 1px solid red;
        background-color: #ffffaf;
        margin: 0px;
    }

    .tooltip {
        position: fixed;
		background-color: #cfffff;
        border: 2px solid #5f5f5f;
        padding: 5px;
		z-index: 120;
        pointer-events: none;
        box-shadow: 0px 0px 5px #000000;
    }
</style>
