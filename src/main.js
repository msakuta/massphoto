'use strict';
var scale = 1.;
var translate = [300, 300];
var focus = null;

const imageContainer = document.getElementById("imageContainer");

const zoomedImage = document.getElementById("zoomedImage");
const zoomedPath = document.getElementById("zoomedPath");

function applyZoom(event, preventDefault=false){
    if(focus === null) return true;
    if(preventDefault)
        event.preventDefault();

    var deltaScale = Math.exp(event.deltaY < 0 ? 0.1 : -0.1);

    var x = event.clientX;
    var y = event.clientY;
    // translate[0] = -x * deltaScale + x;
    // translate[1] = -y * deltaScale + y;

    scale *= deltaScale;

    // Restrict scale
    scale = Math.min(Math.max(1, scale), 20);

    // Apply scale transform
    imageContainer.style.transform = `translate(${translate[0]}px, ${translate[1]}px) scale(${scale})`;
}

function setFocus(image){
    focus = focus !== image ? image : null;
    if(focus){
        zoomedImage.setAttribute("src", `files/${joinPath(rootPath, image.path)}`);
        imageContainer.style.display = "block";
        imageContainer.style.position = "fixed";
        imageContainer.style.left = "0";
        imageContainer.style.top = "0";
        imageContainer.style.transform = `translate(${translate[0]}px, ${translate[1]}px) scale(${scale})`;
        imageContainer.style.zIndex = 100;
        zoomedPath.innerHTML = image.basename;
    }
    else{
        imageContainer.style.display = "none";
    }
}

let selectedImages = {};

async function callAPISingle(src, api, filter = x => x) {
    const regex0 = /\/https*:\/\/\/.+\/t\//;
    const srcWithoutUrl = src.replace(regex0, "");
    const regex = /t\//;
    const regex2 = /e\/t\//;
    try{
        const resp = await fetch(`/${api}/${filter(srcWithoutUrl.replace(regex, "").replace(regex2, ""))}`);
        const text = await resp.text();
        return [resp.status, text];
    }
    catch(e){
        return [500, e];
    }
}

function selectImage(image) {
    const originPath = image.getAttribute("originPath");
    if(originPath in selectedImages){
        delete selectedImages[originPath];
        image.classList.remove("selected");
    }
    else{
        selectedImages[originPath] = true;
        image.classList.add("selected");
    }
}


var dragStart = null;
imageContainer.addEventListener("mouseup", (event) => {
    event.preventDefault();
    if (event.button !== 0) return;
    setFocus(focus);
    return false;
})
imageContainer.addEventListener("mousedown", (event) => {
    event.preventDefault();
    dragStart = [event.clientX, event.clientY];
});
imageContainer.addEventListener("mousemove", (event) => {
    if(dragStart && focus){
        translate[0] += event.clientX - dragStart[0];
        translate[1] += event.clientY - dragStart[1];
        imageContainer.style.transform = `translate(${translate[0]}px, ${translate[1]}px) scale(${scale})`;
        dragStart = [event.clientX, event.clientY];
    }
})
imageContainer.addEventListener("mouseup", (event) => {
    dragStart = null;
});
imageContainer.addEventListener("mouseleave", (event) => {
    dragStart = null;
});
imageContainer.addEventListener("wheel", (event) => applyZoom(event, true));


function applyOnClick(name, callback){
    const button = document.getElementById(`${name}Button`);
    if(button){
        button.onclick = (event) => {
            event.stopPropagation();
            callback();
        }
    }
}

applyOnClick("home", () => {
    rootPath = "";
    loadPage(rootPath);
});
applyOnClick("up", () => {
    const splitPath = rootPath.split("/");
    if(1 < splitPath.length){
        rootPath = splitPath.slice(0, splitPath.length - 1).join("/");
        loadPage(rootPath);
    }
    else{
        rootPath = "";
        loadPage(rootPath);
    }
});
applyOnClick("left", () => {});
applyOnClick("right", () => {});

// stop annoying context menu on right click
document.addEventListener("contextmenu", event => event.preventDefault());

document.body.addEventListener("wheel", applyZoom);

document.body.addEventListener("keydown", (event) => {
    const move = (name) => {
        console.log(name);
        fetch(name, {
            method: "POST",
        })
        .then(() => location.reload());
    }
    switch(event.key){
        case "ArrowLeft":{
            console.log("left");
            let focusIndex = Array.prototype.indexOf.call(images, focus);
            if(0 <= focusIndex){
                setFocus(images[(focusIndex - 1 + images.length) % images.length]);
            }
            break;
        }
        case "ArrowRight":{
            console.log("right");
            let focusIndex = Array.prototype.indexOf.call(images, focus);
            if(0 <= focusIndex){
                setFocus(images[(focusIndex + 1) % images.length]);
            }
            break;
        }
        case "u": move("up"); break;
        case "h": move("left"); break;
        case "k": move("right"); break;
    }
});

let videos = document.getElementsByClassName("video");
Array.prototype.forEach.call(videos, function(image){
    image.addEventListener("mousedown", (event) => {
        event.preventDefault();
        if (event.button === 0) {
            selectImage(image);
            return false;
        }
        if (event.button === 2){
            loadVideo(image.getAttribute('src'));
        }
        return false;
    });
});

async function loadVideo(vidURL){
    const response = await fetch(vidURL, {
        method: 'GET',
    });
    if (response.ok) {
        var videoBlob = await response.blob();
        var vid = URL.createObjectURL(videoBlob); // IE10+

        var myVideo = document.getElementsByTagName('video')[0];
        myVideo.src = vid;
        myVideo.load();
        myVideo.play();
        //video.src = vid;
        const mediaTitle = document.getElementById("mediaTitle");
        mediaTitle.innerHTML = vidURL;
    }
}

let rootPath = "";

window.addEventListener('load', async () => loadPage(rootPath));

function joinPath(root, path){
    if(root === ""){
        return path;
    }
    else {
        return root + "/" + path;
    }
}

async function loadPage(path){
    const res = await fetch(`/file_list/${path}`);
    const json = await res.json();

    const pathElem = document.getElementById("path");
    pathElem.innerHTML = `"${json.path}", ${json.dirs.length} dirs, ${json.files.length} files`;

    const thumbnailsElem = document.getElementById("thumbnails");

    setFocus(null);

    while(thumbnailsElem.firstChild){
        thumbnailsElem.removeChild(thumbnailsElem.firstChild);
    }

    for(let i in json.dirs) {
        const dir = json.dirs[i];
        const thumbContainer = document.createElement("div");
        thumbContainer.className = "dir showcase";
        const captionElem = document.createElement("div");
        captionElem.className = "abs labelText smallText";
        captionElem.innerHTML = dir.file_count;
        thumbContainer.appendChild(captionElem);
        if(dir.image_first){
            const imageElem = document.createElement("img");
            imageElem.src = `/thumbs/${joinPath(rootPath, dir.image_first)}`;
            thumbContainer.appendChild(imageElem);
        }

        thumbContainer.addEventListener("mouseup", (event) => {
            event.preventDefault();
            if (event.button !== 0) return;
            rootPath = dir.path;
            loadPage(rootPath);
        });
        thumbContainer.addEventListener("contextmenu", (event) => {
            event.preventDefault();
        });

        thumbnailsElem.appendChild(thumbContainer);
    }

    for(let i in json.files) {
        const image = json.files[i];
        const thumbContainer = document.createElement("div");
        thumbContainer.className = "dir showcase";
        const captionElem = document.createElement("div");
        captionElem.className = "abs labelText smallText";
        captionElem.innerHTML = image.label;
        thumbContainer.appendChild(captionElem);
        const imageElem = document.createElement("img");
        imageElem.className = "zoomInt";
        imageElem.src = `/thumbs/${joinPath(rootPath, image.path)}`;
        thumbContainer.appendChild(imageElem);

        imageElem.addEventListener("mouseup", (event) => {
            event.preventDefault();
            if (event.button === 0) {
                setFocus(image, imageElem);
            }
            if (event.button === 2){
                selectImage(imageElem);
                return false;
            }
            return false;
        });
        imageElem.addEventListener("contextmenu", (event) => {
            event.preventDefault();
        });

        thumbnailsElem.appendChild(thumbContainer);
    }
}