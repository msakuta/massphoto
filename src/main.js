'use strict';
var scale = 1.;
var translate = [300, 300];
var focus = null;

const dirs = document.getElementsByClassName("dir");
Array.prototype.forEach.call(dirs, function(dir){
    dir.addEventListener("mouseup", (event) => {
        event.preventDefault();
        if (event.which !== 3) return;
        fetch("/path", {
            method: "POST",
            body: dir.getAttribute("path"),
        })
        .then(() => location.reload())
    });
    dir.addEventListener("contextmenu", (event) => {
        event.preventDefault();
    });
})

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
    let imageUrl = null;
    let imageSrc = null;
    for(let i = 0; i < image.childNodes.length; i++){
        const child = image.childNodes[i];
        if(child instanceof HTMLImageElement){
            const src = child.src;
            const regex = /\/files\/t\//;
            const regex2 = /\/files\/e\/t\//;
            imageUrl = src.replace(regex, "/files/").replace(regex2, "/files/e/");
        }
        if(child instanceof HTMLDivElement){
            const regex = /^e\/t\//;
            imageSrc = child.innerHTML.replace(regex, "");
        }
    }
    focus = focus !== image ? image : null;
    if(imageUrl !== null){
        zoomedImage.setAttribute("src", imageUrl);
    }
    if(focus){
        imageContainer.style.display = "block";
        imageContainer.style.position = "fixed";
        imageContainer.style.left = "0";
        imageContainer.style.top = "0";
        imageContainer.style.transform = `translate(${translate[0]}px, ${translate[1]}px) scale(${scale})`;
        imageContainer.style.zIndex = 100;
        zoomedPath.innerHTML = imageSrc;
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

const regexE = /e\//;

const encryptSingle = (src) => callAPISingle(src, "encrypt_file");
const shredSingle = (src) => callAPISingle(src, "shred", x => x.replace(regexE, ""));

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

let images = document.getElementsByClassName("zoomable");
Array.prototype.forEach.call(images, function(image){
    image.addEventListener("mouseup", (event) => {
        event.preventDefault();
        if (event.button === 0) {
            selectImage(image);
            return false;
        }
        if (event.button === 2){
            if(mode === NoMode)
                setFocus(image);
        }
        return false;
    });
    image.addEventListener("contextmenu", (event) => {
        event.preventDefault();
    })
})

var dragStart = null;
imageContainer.addEventListener("mouseup", (event) => {
    event.preventDefault();
    if (event.button !== 2) return;
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


function applyOnClick(name){
    const button = document.getElementById(`${name}Button`);
    if(button){
        button.onclick = (event) => {
            event.stopPropagation();
            fetch(name, {
                method: "POST",
            })
            .then(() => location.reload());
        }
    }
}

applyOnClick("home");
applyOnClick("up");
applyOnClick("left");
applyOnClick("right");

const NoMode = 0;
const EncryptMode = 1;
const ShredMode = 2;
let mode = NoMode;
const encryptButton = document.getElementById("encryptButton");
if(encryptButton){
    encryptButton.onclick = async (event) => {
        event.stopPropagation();
        if(Object.getOwnPropertyNames(selectedImages).length === 0){
            alert("No files are selected");
            return;
        }
        const promises = Object.getOwnPropertyNames(selectedImages).map(encryptSingle);
        const results = await Promise.all(promises);
        alert(`Results:\n${
            results.reduce((acc, cur) => acc + `[${cur[0]}]: ${cur[1]}` + "\n", "")
        }`);
        location.reload(); // Reload to reflect the fact that the file is removed
    };
}

let shredding = false;
const shredButton = document.getElementById("shredButton");
if(shredButton){
    shredButton.onclick = async (event) => {
        event.stopPropagation();
        if(Object.getOwnPropertyNames(selectedImages).length === 0){
            alert("No files are selected");
            return;
        }
        if(!confirm(`Are you sure you want to delete these files?\n${Object.getOwnPropertyNames(selectedImages).reduce((acc, cur) => {
            return acc !== "" ? acc + "\n" + cur : cur;
        }, "")}`)){
            return;
        }
        // Shreading multiple files at the same time may fail because of file renaming, so we need to
        // run them in serial rather than parallel.
        const results = [];
        for(const image of Object.getOwnPropertyNames(selectedImages)){
            results.push(await shredSingle(image));
        }
        alert(`Results:\n${
            results.reduce((acc, cur) => acc + `[${cur[0]}]: ${cur[1]}` + "\n", "")
        }`);
        location.reload(); // Reload to reflect the fact that the file is removed
    };
}

function updateButtons(){
    if(encryptButton)
        encryptButton.style.boxShadow = mode === EncryptMode ? "0px 0px 5px red" : "";
    if(shredButton)
        shredButton.style.boxShadow = mode === ShredMode ? "0px 0px 5px red" : "";
}

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
