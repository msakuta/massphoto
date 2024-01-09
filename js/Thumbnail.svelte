<script>
    import { createEventDispatcher } from 'svelte';
    import { joinPath } from './joinPath';

    const dispatch = createEventDispatcher();

    export let image = {label: "label"};
    export let dir = null;
    export let rootPath = "";
    export let baseUrl = "";


    function imagePath(){
        if(dir){
            if(dir.locked){
                return `${baseUrl}/lock.png`;
            }
            if(!dir.image_first){
                return `${baseUrl}/unknown.png`;
            }
            else{
                return `${baseUrl}/thumbs/${joinPath(rootPath, joinPath(dir.path, dir.image_first))}`;
            }
        }
        else{
            if(image.video){
                return `${baseUrl}/video.png`;
            }
            return `${baseUrl}/thumbs/${joinPath(rootPath, image.path)}`;
        }
    }

    function imageStyle(){
        return `background-image: url(${imagePath()})`;
    }

    function mouseup(event) {
        event.preventDefault();
        if (event.button === 0) {
            dispatch('setFocus', joinPath(rootPath, dir ? dir.path : image.path));
        }
        if (event.button === 2){
            dispatch('selectImage');
            return false;
        }
        return false;
    }

    function contextmenu(event) {
        event.preventDefault();
    }

    function dirStyle() {
        if(dir)
            return `background-image: url(${baseUrl}/directory.png)`;
        else
            return "";
    }
</script>

<div class="dir showcase" style={dirStyle()}>
    <div class="abs labelText smallText">{dir ? dir.path : image.label}</div>
    <div class={dir ? "smallIcon" : "bigIcon"}>
        <div class="zoomInt" style={imageStyle()}
            on:mouseup={mouseup} on:contextmenu={contextmenu} alt={dir ? dir.image_first : image.basename} />
    </div>
</div>

<style>
    .abs {
        position: absolute;
    }
    .dir {
        position: relative;
        padding: 0.1em;
        margin: 0.1em;
        background-color: #7ff;
        border: solid 2px #077;
        background-size: 100%;
    }
    .showcase {
        /* width: 20em; */
        /* height: 30em; */
        width: 100px;
        height: 100px;
        background-color: #F0F8FF;
        padding: 0em;
        margin-bottom: 5px;
    }
    .labelText {
        font-weight: bold;
        text-shadow: 1px 1px #fff, -1px -1px 0 #fff, 1px -1px 0 #fff, -1px 1px 0 #fff;
        overflow-x: hidden;
        pointer-events: none;
        z-index: 10;
    }
    .smallText {
        font-size: 0.7em;
        pointer-events: none;
    }
    .zoomInt {
        margin: auto;
        width: 100%;
        height: 100%;
        background-size: contain;
        background-position: center center;
        background-repeat: no-repeat;
    }
    .smallIcon {
        padding: 15%;
        width: 70%;
        height: 70%;
        position: relative;
    }
    .bigIcon {
        width: 100%;
        height: 100%;
        position: relative;
    }
</style>