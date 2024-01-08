# Photoalbum-rust

A photo album web server application in Rust

## Background

There are tons of online photo services out there.
However, none of them fits my use case, which is on-premise
private photo albums.

The one I used in particular was zenphoto, but it was implemented
in PHP, which is not great for performance.

Also, I was looking for an excuse to exercise practical actix-web
application.

So I decided to make lightweight, fast, self-contained and
easy to maintiain one with Rust.


## Overview

This application uses actix-web as the server, and uses pure
JavaScript hosted by itself.

Notable features:

* It puts priority on performance.
* It respects directory structure of uploaded files to easily organize many images.
* No database configuration is necessary for deployment.

It caches thumbnails to a local database (sqlite via rusqlite)
and quickly preview the list of images.

Using sqlite, the server is almost maintenance-free.
The only maintenance that could be necessary is to remove old caches,
but I doubt even if it's necessary since cached thumbnails are usually
much smaller than the original files.


## TODOs

* [ ] Access control per directory
* [ ] Video playback support (mp4)
* [ ] Periodic update of new file cache
* [ ] Periodic cleanup of outdated cache

## Prerequsites

* Rust 1.74
* npm 8.3.1

## How to run dev server

Run the server in debug mode

```
cargo r -- <path_to_albums_dir>
```

and run the frontend

```
npm ci
npm run dev
```

Now you can browse http://localhost:8080/

In this mode, the frontend source is watched and hot reloaded as they are edited.
However, backend Rust code won't be recompiled.
You could use the `cargo watch` command like below, but it will recompile by any file edits, including the frontend.

```
cargo watch -x "run <path_to_albums_dir>"
```


## How to build the production server

First, build the frontend bundle:

```
npm run build
```

and build the server application:

```
cargo b --release
```

It will produce an executable in target/release including the frontend source code baked into binary.
You can bring it anywhere and simply run it.
