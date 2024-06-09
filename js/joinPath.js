export function joinPath(root, path){
    if(root === ""){
        return path;
    }
    else if(path === "..") {
        const i = root.lastIndexOf("/");
        if (0 < i) {
            return root.substring(0, i);
        }
        return "";
    }
    else {
        return root + "/" + path;
    }
}
