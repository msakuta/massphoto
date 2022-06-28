<script>
    export let image = {label: "label"};
    export let dir = null;
    export let rootPath = "";
    export let baseUrl = "";

    function joinPath(root, path){
        if(root === ""){
            return path;
        }
        else {
            return root + "/" + path;
        }
    }

    function imagePath(){
        if(dir){
            return `${baseUrl}/thumbs/${joinPath(rootPath, dir.image_first)}`;
        }
        else{
            return `${baseUrl}/thumbs/${joinPath(rootPath, image.path)}`;
        }
    }

    function mouseup(event) {
        event.preventDefault();
        if (event.button === 0) {
            setFocus(image, imageElem);
        }
        if (event.button === 2){
            selectImage(imageElem);
            return false;
        }
        return false;
    }

    function contextmenu(event) {
        event.preventDefault();
    }
</script>

<div class="dir showcase">
    <div class="abs labelText smallText">{dir ? dir.file_count : image.label}</div>
    <div>
        <img class="zoomInt" src={imagePath()}
            on:mouseup={mouseup} on:contextmenu={contextmenu} alt={dir ? dir.image_first : image.basename}>
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
    }
    .smallText {
        font-size: 0.7em;
        pointer-events: none;
    }
    .zoomInt {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }
</style>