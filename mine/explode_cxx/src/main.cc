#include <stdint.h>
#include <string.h>
#include <thread>

extern "C" void follow_ip(const char * ip, uint64_t len);
extern "C" void filter_ip(const char * ip, uint64_t len);
extern "C" void safe_loop();

int main(int argc, char** argv) {
	const char* ip = "127.0.0.1";
	follow_ip(ip, strlen(ip));

	//
	const char* in_ip = "127.0.0.1";
	filter_ip(in_ip, strlen(in_ip));


	for (;;) {
		std::this_thread::sleep_for(std::chrono::milliseconds(1000));
		safe_loop();
	}
}