export function joinPath(root, path){
    if(root === ""){
        return path;
    }
    else {
        return root + "/" + path;
    }
}
