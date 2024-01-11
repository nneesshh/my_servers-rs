#!/bin/sh

startwork_release()
{
	cd ../bin/
	echo "begin start server"

  	#export LD_LIBRARY_PATH=".:$LD_LIBRARY_PATH"
	#export LD_PRELOAD="libjemalloc.so.2"

	nohup ./safecomm_server -k abcdefg41 -s 18.163.12.34:29619 --tcp & echo $! > safecomm_server.pid
	
	echo "end start server"
	cd ../script/
}

stopwork_release()
{
    cd ../bin/
    echo "begin stop server"
    #pkill server;
    #kill -9 $(sed -n 1p server.pid);
    kill -9 `cat safecomm_server.pid`
    sleep 2
    echo "end stop server"
    cd ../script/

}

startwork_debug()
{
	cd ../bin/Debug/
	echo "begin start server"

  	#export LD_LIBRARY_PATH=".:$LD_LIBRARY_PATH"
	#export LD_PRELOAD="libjemalloc.so.2"

	nohup ./safecomm_server -k abcdefg41 -s 18.163.12.34:29619 --tcp & echo $! > safecomm_server.pid
	
	echo "end start server"
	cd ../../script/
}

stopwork_debug()
{
    cd ../bin/Debug/
    echo "begin stop server"
    #pkill server;
    #kill -9 $(sed -n 1p server.pid);
    kill -9 `cat safecomm_server.pid`
    sleep 2
    echo "end stop server"
    cd ../../script/

}


    case $1 in
        -d)
            stopwork_debug
	        startwork_debug
        ;;
        -r)
            stopwork_release
	        startwork_release
        ;;
        *)
            stopwork_release
            startwork_release
        ;;
   esac
