<script>
    import { createEventDispatcher } from 'svelte';
    import Element from './MainMenu/Element.svelte';
    import userImage from '../assets/user.png';
    import userAddImage from '../assets/userAdd.png';
    import usersImage from '../assets/users.png';
    import userLogoutImage from '../assets/userLogout.png';
    import keyImage from '../assets/key.png';
    import clearCacheImage from '../assets/clearCache.png';
    import lockImage from '../assets/lock.png';
    import changeOwnerImage from '../assets/changeOwner.png';
    import uploadImage from '../assets/upload.png';
    import deleteImage from '../assets/shred.png';
    import moveDirImage from '../assets/moveDirectory.png';


    const dispatch = createEventDispatcher();

    export let userName = "";
    export let userIsAdmin = false;

    function close() { dispatch('close') }
</script>

<div class="back" on:click={close}>
    <div class="container">
        <Element alt="login" label="Login" src={userImage} on:click={() => dispatch('login')} />
        {#if userName}
            <Element alt="logout" label="Logout" src={userLogoutImage} on:click={() => dispatch('logout')} />
        {/if}
        {#if userIsAdmin}
            <Element alt="userAdd" label="Add User" src={userAddImage} on:click={() => dispatch('userAdd')} />
            <Element alt="userList" label="User List" src={usersImage} on:click={() => dispatch('userList')} />
        {/if}
        {#if userName}
            <Element alt="changePassword" label="Change Password" src={keyImage} on:click={() => dispatch('changePassword')} />
        {/if}
        {#if userIsAdmin}
            <Element alt="clearcache" label="Clear Cache" src={clearCacheImage} on:click={() => dispatch('clearCache')} />
        {/if}
        {#if userName}
            <Element alt="lock" label="Lock Current Album" src={lockImage} on:click={() => dispatch('lock')} />
        {/if}
        {#if userIsAdmin}
            <Element alt="owner change" label="Change Album Owner" src={changeOwnerImage} on:click={() => dispatch('ownerChange')} />
        {/if}
        {#if userIsAdmin}
            <Element alt="upload" label="Upload" src={uploadImage} on:click={() => dispatch('upload')} />
        {/if}
        {#if userIsAdmin}
            <Element alt="delete" label="Delete" src={deleteImage} on:click={() => dispatch('delete')} />
        {/if}
        {#if userIsAdmin}
            <Element alt="move" label="Move Files" src={moveDirImage} on:click={() => dispatch('move')} />
        {/if}
    </div>
</div>


<style>
    .back {
        position: fixed;
        left: 0;
        top: 0;
        width: 100%;
        height: 100%;
        margin: 0;
        background-color: rgba(0, 0, 0, 0.25);
        z-index: 1000;
    }

    .container {
        position: fixed;
        border: 2px solid #3f3f3f;
        top: 52px;
        right: 4px;
        /* width: 80%; */
        max-width: 500px;
        margin: auto;
        padding: 10px;
        background-color: #f3f3f3;
        text-align: left;
        box-shadow: 0px 0px 10px black;
    }
</style>
