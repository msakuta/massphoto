# Massphoto

A photo album web server application in Rust

## Background

There are tons of online photo services out there.
However, none of them fits my use case, which is on-premise
private photo albums.

Long time ago, I used zenphoto, but it was implemented in PHP and used MySQL, which is not great for performance and maintenability.

So I decided to make a lightweight, fast, self-contained and
easy to maintiain one with Rust.


## Overview

This is a web server application that hosts photo albums.
The most similar existing project is [zenphoto](https://www.zenphoto.org/), but aims for easier deployment, management and backups.

Notable features:

* **Self-contained deployment**: you don't need any third party dependencies when you deploy.
* **Single-file executable**: a single binary file contains the backend server, the database engine and the frontend.
* **Performance**: it uses [actix-web](https://actix.rs/) and [svelte](https://svelte.dev/),
  which is one of the fastest combinations of frontend/backend.
* **Easy management**: upload your files to the server directory, and that's it.
  Your photo albums show up on the web app, organized in a directory tree.
* **Zero-config database**: it uses embedded SQLite databse engine, which initializes on the first start.
* **Trivial backup**: since all of your photos and database are in the directory,
  simply copying the directory makes a backup.
* **Cross platform**: if Rust and SQLite compiles, it works on your platform.


## Non-goals

* Fully featured CMS. It's tempting to be ambitious but it would take forever to finish.
* Large scale service. This software is designed for few dozen active users at maximum. SQLite concurrency model limits further scalability. It would require dedicated DBMS and would be more complicated than desired.
* Secure system. Do not keep sensitive data in this server, unless you limit access only from internal network.


## Technical details

It caches thumbnails to a local database and quickly preview the list of images.

Using sqlite, the server is almost maintenance-free.
The only maintenance that could be necessary is to remove old caches,
but I doubt even if it's necessary since cached thumbnails are usually
much smaller than the original files.


## Access Control Rules

It has somewhat different access control rules from usual filesystems. It is inspired by zenphoto, where anyone can unlock and see the album contents if the password is given.

First, user accounts are managed in such a way that:

* Each account has a user name, a password and is_admin flag.
* When the applciation starts for the first time, it will create an admin account with the default name 'admin'.
* Only the admin can create new accounts or delete them.
* User can login to an account by entering a password. The login state is kept as long as the session lives.
* A user can change his/her own password.

And these are the rules of albums:

* Each album has an owner user account and an optional password.
* If an album has a password, it cannot be seen by users except the owner or the admins.
* The owner or the admin can set a password to an album.
* Users can see the contents of an album if they give a correct password, even if they do not login.


## TODOs

* [x] Access control per directory
* [x] Description for each image
* [x] Video playback support (mp4)
* [x] Upload images
* [x] Deleting images
* [x] Moving images
* [ ] Creating albums
* [ ] Deleting albums
* [ ] Moving albums
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
