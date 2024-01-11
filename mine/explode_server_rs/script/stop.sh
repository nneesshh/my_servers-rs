#!/bin/sh

stopwork_release()
{
    cd ../bin/

    echo "begin stop server"
    #pkill server;
    #kill -9 $(sed -n 1p server.pid);
    kill -9 `cat safecomm_server.pid`

    echo "end stop server"
    cd ../script/

}


stopwork_debug()
{
    cd ../bin/Debug/
    echo "begin stop server"
    
    #pkill server;
    #kill -9 $(sed -n 1p server.pid);
    kill -9 `cat safecomm_server.pid`
 
    echo "end stop server"
    cd ../../script/

}


    case $1 in
                 -d)
             stopwork_debug
             ;;
                 -r)
             stopwork_release
             ;;
            *)
            stopwork_release
        ;;
   esac
