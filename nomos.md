- [ ] (X) Improve error handling :: incoprorate nemesis
- [ ] (Z) Move away from actual files to virtual files ::
    * Could even try to work with env vars as well (not global, but scoped to the program or something)
- [ ] No lockfile :: 
    * I could limit the amount of connections by introducing another server signal channel (e.g. server-lock, handler-connected) that is created when a server connects and persists until it disconnects.
    * To attempt to handle connecting more than one client, I really need to introduce a client signal channel called in-use or similar. This way I could detect that before overwriting another possible request.xff made from another thread / program / whatever.
