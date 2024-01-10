#include <stdint.h>
#include <string.h>

extern "C" int follow_ip(const uint8_t * ip, uint64_t len);
extern "C" int filter_ip(const uint8_t * ip, uint64_t len);

int main(int argc, char** argv) {
	const char* ip = "127.0.0.1";
	follow_ip(ip, strlen(ip));
	filter_ip(ip, strlen(ip));
}