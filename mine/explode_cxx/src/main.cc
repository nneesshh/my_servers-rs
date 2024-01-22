#include <stdint.h>
#include <string.h>
#include <thread>

#include "rust/filter.rs.h"

int main(int argc, char** argv) {
	const char* ip = "127.0.0.1";
	follow_ip(ip);

	//
	const char* in_ip = "127.0.0.1";
	filter_ip(in_ip);

	//
	for (;;) {
		std::this_thread::sleep_for(std::chrono::milliseconds(1000));
		safe_loop();
	}
}