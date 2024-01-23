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
	const char* s = "123,456|789";
	auto vec = string_to_vec(s, ",");
	for (auto s : vec) {
		printf("s=%s \r\n", s.c_str());
		printf("n=%d \r\n", string_to_u64(s.c_str()));
	}

	//
	for (;;) {
		std::this_thread::sleep_for(std::chrono::milliseconds(1000));
		safe_loop();
	}
}