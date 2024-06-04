/*
Thread pools are used for connection handling for login procedures or status requests, thereby preventing
DOS attacks. After the login of a player has completed, a new dedicated thread is spawned for each 
player.

The idea of this thread pool came from the rust book guide on a multithreaded web server.

References:

//TODO: is there an open source project that achieves the same goals?
*/